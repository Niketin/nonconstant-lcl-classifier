use clap::{values_t_or_exit, App, Arg};
use console::style;
use indicatif::ProgressBar;
use itertools::Itertools;
use log::info;
use std::time::Instant;
use thesis_tool_lib::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize env_logger.
    env_logger::init();

    // Create new command line program.
    let matches = App::new("Thesis tool")
        .about("This tool can be used to find negative proofs of LCL-problems solvability on the Port Numbering model.")
        .arg(Arg::with_name("graph_size_bound")
            .long("graph_sizes")
            .short("n")
            .takes_value(true)
            .number_of_values(2)
            .help("Set bounds for graph sizes. The range is inclusive. First value sets the lower bound and second value sets the upper bound of vertices.")
            .required(true)
        )
        .arg(Arg::with_name("active_configurations")
            .short("A")
            .help("Sets the active configurations of the LCL-problem.")
            .takes_value(true)
            .min_values(1)
            .required(true)
        )
        .arg(Arg::with_name("passive_configurations")
            .short("P")
            .help("Sets the passive configurations of the LCL-problem.")
            .takes_value(true)
            .min_values(1)
            .required(true)
        )
        .arg(Arg::with_name("simple_graphs_only")
            .help("Generate only simple graphs.")
            .short("s")
            .long("simple-graphs-only")
            .required(false)
        )
        .arg(Arg::with_name("progress")
            .help("Show progress.")
            .short("p")
            .long("show-progress")
            .required(false)
        )
        .arg(Arg::with_name("output_svg")
            .help("If unsatisfiable result is foun, output graph as svg to the path.")
            .long("svg")
            .takes_value(true))
        .get_matches();

    let (n_lower, n_upper) = {
        let values = values_t_or_exit!(matches, "graph_size_bound", usize);
        (values[0], values[1])
    };

    let a = matches
        .values_of("active_configurations")
        .expect("Parsing parameter 'a' failed.")
        .join("\n");
    let p = matches
        .values_of("passive_configurations")
        .expect("Parsing parameter 'p' failed.")
        .join("\n");

    let simple_graphs_only = matches.is_present("simple_graphs_only");
    let show_progress = matches.is_present("progress");
    let get_progress_bar = |n: u64| {
        if show_progress {
            ProgressBar::new(n)
        } else {
            ProgressBar::hidden()
        }
    };

    let graph_generator = if simple_graphs_only {
        BiregularGraph::generate_simple
    } else {
        BiregularGraph::generate_multigraph
    };

    let lcl_problem = LclProblem::new(&a, &p).expect("Parsing the LCL problem failed.");
    let a_len = lcl_problem.active.get_labels_per_configuration();
    let p_len = lcl_problem.passive.get_labels_per_configuration();

    for (i, n) in (n_lower..=n_upper).enumerate() {
        println!(
            "{} Starting the routine for graphs of size {}...",
            style(format!("[{}/{}]", i + 1, n_upper - n_lower + 1))
                .bold()
                .dim(),
            style(format!("n={}", n)).cyan()
        );

        // Generate biregular graphs.
        let now = Instant::now();
        println!(
            "    {} Generating nonisomorphic ({},{})-biregular graphs...",
            style("[1/4]").bold().dim(),
            a_len,
            p_len,
        );
        let graphs = graph_generator(n, a_len, p_len);
        info!(
            "Generated {} nonisomorphic biregular graphs in {} s",
            graphs.len(),
            now.elapsed().as_secs_f32()
        );

        // Encode graphs and LCL-problem into SAT problems.
        let now = Instant::now();
        println!(
            "    {} Creating SAT encoders...",
            style("[2/4]").bold().dim(),
        );
        let pb = get_progress_bar(graphs.len() as u64);
        let encoders = pb
            .wrap_iter(graphs.into_iter())
            .map(|graph| SatEncoder::new(&lcl_problem, graph))
            .collect_vec();
        pb.finish_and_clear();

        println!(
            "    {} Encoding problems and graphs into SAT problems...",
            style("[3/4]").bold().dim(),
        );
        let pb = get_progress_bar(encoders.len() as u64);
        let encodings = pb
            .wrap_iter(encoders.iter())
            .map(|encoder| encoder.encode())
            .collect_vec();
        pb.finish_and_clear();
        info!(
            "Encoded {} SAT problems in {} s",
            encodings.len(),
            now.elapsed().as_secs_f32()
        );

        // Solve SAT problems.
        let now = Instant::now();
        println!(
            "    {} Solving SAT problems...",
            style("[4/4]").bold().dim(),
        );

        let mut result_i = None;
        let pb = get_progress_bar(encodings.len() as u64);
        for (i, encoding) in encodings.iter().enumerate() {
            let result = SatSolver::solve(&encoding);
            pb.inc(1);
            if result == SatResult::Unsatisfiable {
                result_i = Some(i);
                break;
            }
        }
        pb.finish_and_clear();

        if result_i.is_some() {
            println!("    {}", style("An unsatisfiable result found!").green());
        } else {
            println!("    {}", style("No unsatisfiable results.").red());
        }

        info!(
            "Time used for solving SAT problems is {} s",
            now.elapsed().as_secs_f32()
        );
        if result_i.is_some() {
            let graph = encoders[result_i.unwrap()].get_graph();
            let dot = graph.graph.get_dot();
            println!("{}", dot);

            if let Some(path) = matches.value_of("output_svg") {
                save_as_svg(path, &dot).expect("Failed to save graph as svg.");
                println!("{} '{}'", style("Saved the graph to path").green(), path);
            }
            break;
        }
    }

    Ok(())
}
