use clap::{value_t_or_exit, App, Arg};
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
        .arg(Arg::with_name("n")
            .help("Sets the number of vertices in the graphs.")
            .index(1)
            .required(true)
        )
        .arg(Arg::with_name("active_configurations")
            .help("Sets the active configurations of the LCL-problem.")
            .index(2)
            .required(true)
        )
        .arg(Arg::with_name("passive_configurations")
            .help("Sets the passive configurations of the LCL-problem.")
            .index(3)
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
        .get_matches();

    let n = value_t_or_exit!(matches, "n", usize);
    let a = matches
        .value_of("active_configurations")
        .expect("Parsing parameter 'a' failed.");
    let p = matches
        .value_of("passive_configurations")
        .expect("Parsing parameter 'p' failed.");

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

    let lcl_problem = LclProblem::new(a, p).expect("Parsing the LCL problem failed.");
    let a_len = lcl_problem.active.get_labels_per_configuration();
    let p_len = lcl_problem.passive.get_labels_per_configuration();

    // Generate biregular graphs.
    let now = Instant::now();
    println!(
        "{} Generating nonisomorphic ({},{})-biregular graphs of size n={}...",
        style("[1/3]").bold().dim(),
        a_len,
        p_len,
        n
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
        "{} Encoding problems and graphs into SAT problems...",
        style("[2/3]").bold().dim(),
    );
    let pb = get_progress_bar(graphs.len() as u64);
    let encodings = pb
        .wrap_iter(graphs.into_iter())
        .map(|graph| SatEncoder::new(lcl_problem.clone(), graph).encode())
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
        "{} Solving SAT problems...",
        style("[3/3]").bold().dim(),
    );

    let mut found = false;
    let pb = get_progress_bar(encodings.len() as u64);
    for encoding in encodings {
        let result = SatSolver::solve(&encoding);
        pb.inc(1);
        if result == SatResult::Unsatisfiable {
            found = true;
            break;
        }
    }
    pb.finish_and_clear();

    if found {
        println!("An unsatisfiable result found!");
    } else {
        println!("No unsatisfiable results.");
    }

    info!(
        "Time used for solving SAT problems is {} s",
        now.elapsed().as_secs_f32()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcl_on_n4_graphs_unsatisfiable() -> Result<(), Box<dyn std::error::Error>> {
        let n = 4;
        let a = "S S";
        let p = "K K";
        let lcl_problem = LclProblem::new(a, p)?;
        let deg_a = lcl_problem.active.get_labels_per_configuration();
        let deg_p = lcl_problem.passive.get_labels_per_configuration();

        let graphs = BiregularGraph::generate_simple(n, deg_a, deg_p);

        assert!(!graphs.is_empty());

        graphs.into_iter().for_each(|graph| {
            let sat_encoder = SatEncoder::new(lcl_problem.clone(), graph);
            let clauses = sat_encoder.encode();
            let result = SatSolver::solve(&clauses);
            assert_eq!(result, SatResult::Unsatisfiable);
        });

        Ok(())
    }

    #[test]
    fn test_lcl_on_n4_graphs_satisfiable() -> Result<(), Box<dyn std::error::Error>> {
        let n = 5;

        let a = "M U U\nP P P";
        let p = "M M\nP U\nU U";
        let lcl_problem = LclProblem::new(a, p)?;
        let deg_a = lcl_problem.active.get_labels_per_configuration();
        let deg_p = lcl_problem.passive.get_labels_per_configuration();

        let graphs = BiregularGraph::generate_multigraph(n, deg_a, deg_p);

        assert!(!graphs.is_empty());
        graphs.into_iter().for_each(|graph| {
            let sat_encoder = SatEncoder::new(lcl_problem.clone(), graph);
            let clauses = sat_encoder.encode();
            let result = SatSolver::solve(&clauses);
            assert_eq!(result, SatResult::Satisfiable);
        });

        Ok(())
    }

    #[test]
    fn test_lcl_on_n10_graphs_unsatisfiable() -> Result<(), Box<dyn std::error::Error>> {
        let n = 10;

        let a = "M U U\nP P P";
        let p = "M M\nP U\nU U";
        let lcl_problem = LclProblem::new(a, p)?;
        let deg_a = lcl_problem.active.get_labels_per_configuration();
        let deg_p = lcl_problem.passive.get_labels_per_configuration();

        let graphs = BiregularGraph::generate_multigraph(n, deg_a, deg_p);

        assert!(!graphs.is_empty());

        let mut results = graphs.into_iter().map(|graph| {
            let sat_encoder = SatEncoder::new(lcl_problem.clone(), graph);
            let clauses = sat_encoder.encode();
            SatSolver::solve(&clauses)
        });

        // At least one result is unsatisfiable.
        assert!(results.any(|result| { result == SatResult::Unsatisfiable }));

        Ok(())
    }
}
