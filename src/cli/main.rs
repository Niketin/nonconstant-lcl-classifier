mod app;

use app::build_cli;
use clap::values_t_or_exit;
use clap::value_t_or_exit;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use log::info;
use std::time::Instant;
use thesis_tool_lib::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let matches = build_cli().get_matches();

    let matches_find = matches.subcommand_matches("find").unwrap();
    let verbosity = matches_find.occurrences_of("verbosity");
    let progress = matches_find.occurrences_of("progress");
    let n_lower = value_t_or_exit!(matches_find, "min_nodes", usize);
    let n_upper = value_t_or_exit!(matches_find, "max_nodes", usize);

    let get_progress_bar = |n: u64, progress_level| {
        if progress >= progress_level {
            ProgressBar::new(n)
        } else {
            ProgressBar::hidden()
        }
    };

    let get_progress_style = || {
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:20.cyan/blue} {per_sec:>10} {pos:>7}/{len:7} {msg}")
            .progress_chars("##-")
    };

    let get_progress_style_no_speed = || {
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:20.cyan/blue}            {pos:>7}/{len:7} {msg}")
            .progress_chars("##-")
    };

    let get_spinner = || {
        ProgressStyle::default_spinner()
            .template("[{elapsed_precise}] {spinner:47.cyan/blue} {msg}")
    };

    let pb_gen_problems = get_progress_bar(0, 1);
    pb_gen_problems.set_style(get_spinner());
    pb_gen_problems.enable_steady_tick(100);

    pb_gen_problems.set_message("Defining problem(s)");

    // Read a problem or generate class of problems.
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
    pb_gen_problems.set_length(problems.len() as u64);
    pb_gen_problems.set_style(get_progress_style_no_speed());
    pb_gen_problems.finish_with_message("Defining problem(s) done!");

    let mut results = vec![];

    let pb_problems = get_progress_bar(problems.len() as u64, 1);

    pb_problems.set_style(get_progress_style());

    pb_problems.set_message("Trying to find a lower bound proof for each problem...");
    if progress == 1 {
        pb_problems.enable_steady_tick(100);
    }

    for (problem_i, problem) in pb_problems.wrap_iter(problems.iter()).enumerate() {
        let a_len = problem.active.get_labels_per_configuration();
        let p_len = problem.passive.get_labels_per_configuration();

        if verbosity >= 1 {
            println!(
                "Finding for problem {}...",
                style(format!("[{}/{}]", problem_i + 1, problems.len()))
                    .bold()
                    .dim(),
            );
        }

        let indent_level = 2;

        'graph_size_loop: for (i, n) in (n_lower..=n_upper).enumerate() {
            if verbosity >= 2 {
                println!(
                    "{}{} Starting the routine for graphs of size {}...",
                    indent(indent_level),
                    style(format!("[{}/{}]", i + 1, n_upper - n_lower + 1))
                        .bold()
                        .dim(),
                    style(format!("n={}", n)).cyan(),
                );
            }
            let pb = get_progress_bar(0, 2);
            pb.set_style(get_spinner());
            pb.set_message(format!(
                "Generating nonisomorphic ({},{})-biregular graphs...",
                a_len, p_len,
            ));
            pb.enable_steady_tick(100);

            // Generate biregular graphs.
            let now = Instant::now();
            if verbosity >= 3 {
                println!(
                    "{}{} Generating nonisomorphic ({},{})-biregular graphs...",
                    indent(indent_level + 2),
                    style("[1/4]").bold().dim(),
                    a_len,
                    p_len,
                );
            }
            let graphs = BiregularGraph::generate_multigraph(n, a_len, p_len);
            info!(
                "Generated {} nonisomorphic biregular graphs in {} s",
                graphs.len(),
                now.elapsed().as_secs_f32()
            );

            pb.finish_and_clear();

            // Create SAT encoders.
            let now = Instant::now();
            if verbosity >= 3 {
                println!(
                    "{}{} Creating SAT encoders...",
                    indent(indent_level + 2),
                    style("[2/4]").bold().dim(),
                );
            }

            let pb = get_progress_bar(graphs.len() as u64, 2);
            pb.set_style(get_progress_style());
            pb.set_message("Creating SAT encoders");
            let encoders = pb
                .wrap_iter(graphs.into_iter())
                .map(|graph| SatEncoder::new(&problem, graph))
                .collect_vec();

            pb.finish_and_clear();

            // Encode graphs and LCL-problem into SAT problems.
            if verbosity >= 3 {
                println!(
                    "{}{} Encoding problems and graphs into SAT problems...",
                    indent(indent_level + 2),
                    style("[3/4]").bold().dim(),
                );
            }

            let pb = get_progress_bar(encoders.len() as u64, 2);
            pb.set_style(get_progress_style());
            pb.set_message("Encoding problems and graphs into SAT problems");

            let encodings = pb
                .wrap_iter(encoders.iter())
                .map(|encoder| encoder.encode())
                .collect_vec();
            info!(
                "Encoded {} SAT problems in {} s",
                encodings.len(),
                now.elapsed().as_secs_f32()
            );

            // Solve SAT problems.
            let now = Instant::now();
            if verbosity >= 3 {
                println!(
                    "{}{} Solving SAT problems...",
                    indent(indent_level + 2),
                    style("[4/4]").bold().dim(),
                );
            }
            pb.finish_and_clear();

            let mut unsat_result_index = None;

            let pb = get_progress_bar(encodings.len() as u64, 2);
            pb.set_style(get_progress_style());
            pb.set_message("Solving SAT problems");
            pb.set_length(encoders.len() as u64);
            for (i, encoding) in encodings.iter().enumerate() {
                let result = SatSolver::solve(&encoding);
                pb.inc(1);
                if result == SatResult::Unsatisfiable {
                    unsat_result_index = Some(i);
                    break;
                }
            }

            if verbosity >= 3 {
                if unsat_result_index.is_some() {
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
            }

            info!(
                "Time used for solving SAT problems is {} s",
                now.elapsed().as_secs_f32()
            );
            if unsat_result_index.is_some() {
                let graph = encoders[unsat_result_index.unwrap()].get_graph();
                let dot = graph.graph.get_dot();

                results.push((problem.clone(), graph.graph.node_count()));

                if let Some(path) = matches_find.value_of("output_svg") {
                    save_as_svg(path, &dot).expect("Failed to save graph as svg.");
                    if verbosity >= 2 {
                        println!("{} '{}'", style("Saved the graph to path").green(), path);
                    }
                }
                if !matches_find.is_present("all") {
                    break 'graph_size_loop;
                }
            }
        }
    }

    pb_problems.finish_with_message("Finding lower bound proofs done!");

    for (problem, graph_node_count) in results {
        println!("n={:2}: {}", graph_node_count, problem.to_string());
    }

    Ok(())
}

fn indent(level: usize) -> String {
    format!("{:<1$}", "", level)
}
