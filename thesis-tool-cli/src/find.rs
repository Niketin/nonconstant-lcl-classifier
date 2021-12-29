use crate::from_lcl_classifier::fetch_problems;
use clap::{value_t_or_exit, ArgMatches};
use console::style;
use indicatif::{ParallelProgressIterator, ProgressFinish};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use log::info;
use rayon::prelude::*;
use std::{path::PathBuf, str::FromStr, time::Instant};
use thesis_tool_lib::lcl_problem::{Normalizable, Purgeable};
use thesis_tool_lib::{
    caches::{GraphSqliteCache, LclProblemSqliteCache},
    save_as_svg, BiregularGraph, DotFormat, LclProblem, SatEncoder, SatResult, SatSolver,
};

pub fn find(matches_find: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let verbosity = matches_find.occurrences_of("verbosity");
    let progress = matches_find.occurrences_of("progress");
    let n_lower = value_t_or_exit!(matches_find, "min_nodes", usize);
    let n_upper = value_t_or_exit!(matches_find, "max_nodes", usize);

    let sqlite_cache_path = matches_find.value_of("sqlite_cache");

    let mut graph_cache = if sqlite_cache_path.is_some() {
        Some(GraphSqliteCache::new(
            PathBuf::from_str(sqlite_cache_path.unwrap())
                .expect("Database at the given path does not exist"),
        ))
    } else {
        None
    };

    let mut problem_cache = if sqlite_cache_path.is_some() {
        Some(LclProblemSqliteCache::new(
            PathBuf::from_str(sqlite_cache_path.unwrap())
                .expect("Database at the given path does not exist"),
        ))
    } else {
        None
    };

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
            .template("[{elapsed_precise}] {spinner:31.cyan/blue} {pos:>7}/{len:7} {msg}")
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
            let active_degree = value_t_or_exit!(sub_m, "active_degree", usize);
            let passive_degree = value_t_or_exit!(sub_m, "passive_degree", usize);
            let label_count = value_t_or_exit!(sub_m, "label_count", usize);
            LclProblem::get_or_generate_normalized(
                active_degree,
                passive_degree,
                label_count as u8,
                problem_cache.as_mut(),
            )
        }
        ("from_classifier", Some(sub_m)) => {
            let active_degree = value_t_or_exit!(sub_m, "active_degree", i16);
            let passive_degree = value_t_or_exit!(sub_m, "passive_degree", i16);
            let label_count = value_t_or_exit!(sub_m, "label_count", i16);
            let db_path = sub_m.value_of("database_path").unwrap();
            let mut problems = fetch_problems(db_path, active_degree, passive_degree, label_count)
                .expect(
                    format!(
                        "Failed to fetch problems from lcl classifier database at {}",
                        db_path
                    )
                    .as_str(),
                );

            if sub_m.is_present("purge") {
                let old_count = problems.len();
                problems = problems.purge();
                pb_gen_problems.println(format!(
                    "Purging removed {} problems",
                    old_count - problems.len()
                ));
            }

            if sub_m.is_present("normalize") {
                let old_count = problems.len();
                problems = problems.normalize();
                pb_gen_problems.println(format!(
                    "Normalizing removed {} problems",
                    old_count - problems.len()
                ));
            }

            problems
        }
        (_, _) => unreachable!(),
    };
    pb_gen_problems.set_length(problems.len() as u64);
    pb_gen_problems.set_style(get_progress_style_no_speed());
    pb_gen_problems.finish_with_message("Defining problem(s) done!");

    let deg_a = problems
        .first()
        .unwrap()
        .active
        .get_labels_per_configuration();
    let deg_p = problems
        .first()
        .unwrap()
        .passive
        .get_labels_per_configuration();

    let mut graphs = vec![];

    let pb_graphs = get_progress_bar(0, 1);
    pb_graphs.set_style(get_spinner());
    pb_graphs.set_message(format!(
        "Generating nonisomorphic ({},{})-biregular graphs...",
        deg_a, deg_p,
    ));
    pb_graphs.enable_steady_tick(100);

    if verbosity >= 1 {
        eprintln!(
            "Generating nonisomorphic ({},{})-biregular graphs...",
            deg_a, deg_p,
        );
    }
    for n in n_lower..=n_upper {
        // Generate biregular graphs.
        let now = Instant::now();
        let graphs_n = BiregularGraph::get_or_generate(n, deg_a, deg_p, graph_cache.as_mut());
        info!(
            "Generated {} nonisomorphic biregular graphs in {} s",
            graphs_n.len(),
            now.elapsed().as_secs_f32()
        );

        graphs.push(graphs_n);
    }
    let graph_count: usize = graphs.iter().map(|x| x.len()).sum();
    pb_graphs.set_length(graph_count as u64);
    pb_graphs.finish_with_message(format!(
        "Generating nonisomorphic ({},{})-biregular graphs done!",
        deg_a, deg_p,
    ));

    let pb_problems = get_progress_bar(problems.len() as u64, 1);
    pb_problems.set_style(get_progress_style().on_finish(ProgressFinish::WithMessage(
        std::borrow::Cow::Owned("Finding lower bound proofs done!".to_string()),
    )));
    pb_problems.set_message("Trying to find a lower bound proof for each problem...");
    if progress == 1 {
        pb_problems.enable_steady_tick(100);
    }

    let results: Vec<(LclProblem, usize)> = problems
        .par_iter()
        .progress_with(pb_problems)
        .enumerate()
        .flat_map(|(problem_i, problem)| {
            if verbosity >= 1 {
                eprintln!(
                    "Finding for problem {}...",
                    style(format!("[{}/{}]", problem_i + 1, problems.len()))
                        .bold()
                        .dim(),
                );
            }

            let indent_level = 2;

            let mut results = vec![];

            'graph_size_loop: for (i, n) in (n_lower..=n_upper).enumerate() {
                let graphs_n = &graphs[i];
                if verbosity >= 2 {
                    eprintln!(
                        "{}{} Starting the routine for graphs of size {}...",
                        indent(indent_level),
                        style(format!("[{}/{}]", i + 1, n_upper - n_lower + 1))
                            .bold()
                            .dim(),
                        style(format!("n={}", n)).cyan(),
                    );
                }
                // Create SAT encoders.
                let now = Instant::now();
                if verbosity >= 3 {
                    eprintln!(
                        "{}{} Creating SAT encoders...",
                        indent(indent_level + 2),
                        style("[2/4]").bold().dim(),
                    );
                }

                let pb = get_progress_bar(graphs_n.len() as u64, 2);
                pb.set_style(get_progress_style());
                pb.set_message("Creating SAT encoders");
                let encoders = pb
                    .wrap_iter(graphs_n.into_iter())
                    .map(|graph| SatEncoder::new(&problem, graph.clone())) // TODO use immutable reference instead of cloning.
                    .collect_vec();

                pb.finish_and_clear();

                // Encode graphs and LCL-problem into SAT problems.
                if verbosity >= 3 {
                    eprintln!(
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
                    eprintln!(
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
                        eprintln!(
                            "{}{}",
                            indent(indent_level + 2),
                            style("A lower bound found!").green()
                        );
                    } else {
                        eprintln!(
                            "{}{}",
                            indent(indent_level + 2),
                            style("No lower bound found.").red()
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
                            eprintln!("{} '{}'", style("Saved the graph to path").green(), path);
                        }
                    }
                    if !matches_find.is_present("all") {
                        break 'graph_size_loop;
                    }
                }
            }
            results
        })
        .collect();

    for (problem, graph_node_count) in &results {
        println!("n = {:2}: {}", graph_node_count, problem.to_string());
    }

    if matches_find.is_present("print_stats") {
        eprintln!(
            "Found new lower bounds for {}/{} problems",
            results.len(),
            problems.len()
        );

        let sizes = results
            .iter()
            .map(|(_, n)| *n)
            .unique()
            .sorted()
            .collect_vec();

        for n in sizes {
            let count = results.iter().filter(|(_, size)| n == *size).count();
            eprintln!("n = {:2}; count = {:5}", n, count);
        }
    }

    Ok(())
}

fn indent(level: usize) -> String {
    format!("{:<1$}", "", level)
}
