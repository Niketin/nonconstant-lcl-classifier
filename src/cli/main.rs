mod app;

use app::build_cli;
use clap::values_t_or_exit;
use console::style;
use indicatif::ProgressBar;
use itertools::Itertools;
use log::info;
use std::time::Instant;
use thesis_tool_lib::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let matches = build_cli().get_matches();

    let matches_find = matches.subcommand_matches("find").unwrap();
    let (n_lower, n_upper) = {
        let values = values_t_or_exit!(matches_find, "graph_size_bound", usize);
        (values[0], values[1])
    };

    let simple_graphs_only = matches_find.is_present("simple_graphs_only");
    let show_progress = matches_find.is_present("progress");
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

    let problems = match matches_find.subcommand() {
        ("single", Some(sub_m)) => {
            let a = sub_m
                .values_of("active_configurations")
                .expect("Parsing parameter 'a' failed.")
                .join("\n");
            let p = sub_m
                .values_of("passive_configurations")
                .expect("Parsing parameter 'p' failed.")
                .join("\n");
            let lcl_problem = LclProblem::new(&a, &p).expect("Parsing the LCL problem failed.");
            vec![lcl_problem]
        }
        ("class", Some(sub_m)) => {
            let (active_degree, passive_degree, label_count) = {
                let values = values_t_or_exit!(sub_m, "problem_class", usize);
                (values[0], values[1], values[2])
            };
            LclProblem::generate_normalized(active_degree, passive_degree, label_count as u8)
        }
        (_, _) => unreachable!(),
    };

    'problem_loop: for (problem_i, problem) in problems.iter().enumerate() {
        let a_len = problem.active.get_labels_per_configuration();
        let p_len = problem.passive.get_labels_per_configuration();
        println!("problem {}", problem_i);
        println!(
            "{} Finding...",
            style(format!("[{}/{}]", problem_i + 1, problems.len()))
                .bold()
                .dim(),
        );

        let indent_level = 2;
        for (i, n) in (n_lower..=n_upper).enumerate() {
            println!(
                "{}{} Starting the routine for graphs of size {}...",
                indent(indent_level),
                style(format!("[{}/{}]", i + 1, n_upper - n_lower + 1))
                    .bold()
                    .dim(),
                style(format!("n={}", n)).cyan(),
            );

            // Generate biregular graphs.
            let now = Instant::now();
            println!(
                "{}{} Generating nonisomorphic ({},{})-biregular graphs...",
                indent(indent_level + 2),
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

            // Create SAT encoders.
            let now = Instant::now();
            println!(
                "{}{} Creating SAT encoders...",
                indent(indent_level + 2),
                style("[2/4]").bold().dim(),
            );
            let pb = get_progress_bar(graphs.len() as u64);
            let encoders = pb
                .wrap_iter(graphs.into_iter())
                .map(|graph| SatEncoder::new(&problem, graph))
                .collect_vec();
            pb.finish_and_clear();

            // Encode graphs and LCL-problem into SAT problems.
            println!(
                "{}{} Encoding problems and graphs into SAT problems...",
                indent(indent_level + 2),
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
                "{}{} Solving SAT problems...",
                indent(indent_level + 2),
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
                println!(
                    "{}{}",
                    indent(indent_level + 2),
                    style("An unsatisfiable result found!").green()
                );
            } else {
                println!(
                    "{}{}",
                    indent(indent_level + 2),
                    style("No unsatisfiable results.").red()
                );
            }

            info!(
                "Time used for solving SAT problems is {} s",
                now.elapsed().as_secs_f32()
            );
            if result_i.is_some() {
                let graph = encoders[result_i.unwrap()].get_graph();
                let dot = graph.graph.get_dot();
                println!("{}", dot);
                println!("{:?}", problem);

                if let Some(path) = matches.value_of("output_svg") {
                    save_as_svg(path, &dot).expect("Failed to save graph as svg.");
                    println!("{} '{}'", style("Saved the graph to path").green(), path);
                }
                if !matches_find.is_present("all") {
                    break 'problem_loop;
                }
            }
        }
    }

    Ok(())
}

fn indent(level: usize) -> String {
    format!("{:<1$}", "", level)
}
