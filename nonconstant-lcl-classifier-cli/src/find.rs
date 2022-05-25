use crate::from_stdin::from_stdin;
use clap::{value_t_or_exit, ArgMatches};
use indicatif::{ParallelProgressIterator, ProgressFinish};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use log::info;
use nonconstant_lcl_classifier_lib::{
    caches::{GraphSqliteCache, LclProblemSqliteCache},
    save_as_svg, BiregularGraph, DotFormat, LclProblem, SatEncoder, SatResult, SatSolver,
};
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::{path::PathBuf, str::FromStr, time::Instant};

pub fn find(matches_find: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let progress = matches_find.occurrences_of("progress");
    let n_lower = value_t_or_exit!(matches_find, "min_nodes", usize);
    let n_upper = value_t_or_exit!(matches_find, "max_nodes", usize);

    let sqlite_cache_path = matches_find.value_of("sqlite_cache");

    let mut graph_cache = sqlite_cache_path.map(|path| {
        GraphSqliteCache::new(
            PathBuf::from_str(path)
                .expect("Database at the given path does not exist")
                .as_path(),
        )
    });

    let mut problem_cache = sqlite_cache_path.map(|path| {
        LclProblemSqliteCache::new(PathBuf::from_str(path).expect("Invalid path").as_path())
    });

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

    let now = Instant::now();
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
        ("from_stdin", Some(sub_m)) => {
            let no_ignore_solved = sub_m.is_present("no_ignore");
            let problems =
                from_stdin(!no_ignore_solved).expect("Failed to read problems from stdin");
            assert!(!problems.is_empty(), "No problems were given to stdin",);
            problems
        }
        (_, _) => unreachable!(),
    };
    let time_problems = now.elapsed().as_secs_f32();
    pb_gen_problems.set_length(problems.len() as u64);
    pb_gen_problems.set_style(get_progress_style_no_speed());
    pb_gen_problems.finish_with_message("Defining problem(s) done!");

    // Assume all active partitions have same degree.
    let deg_a = problems
        .first()
        .unwrap()
        .active
        .get_labels_per_configuration();

    // Assume all passive partitions have same degree.
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

    let now = Instant::now();
    for n in n_lower..=n_upper {
        // Get biregular graphs from cache or generate them.
        let graphs_n = BiregularGraph::get_or_generate(n, deg_a, deg_p, graph_cache.as_mut());
        graphs.push(graphs_n);
    }
    let time_graphs = now.elapsed().as_secs_f32();
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
    let now = Instant::now();
    let results: Vec<(LclProblem, usize)> = problems
        .par_iter()
        .progress_with(pb_problems)
        .flat_map(|problem| {
            let mut results = vec![];

            'graph_size_loop: for graphs_n in &graphs {
                // Create SAT encoder iterator.
                let encoders = graphs_n
                    .iter()
                    .enumerate()
                    .map(|(graph_index, graph)| (graph_index, SatEncoder::new(problem, graph.clone()))); // TODO use immutable reference instead of cloning.

                let mut found = 0;

                // Solve SAT problems.
                'encoder_loop: for (graph_index, encoder) in encoders {
                    let result = SatSolver::solve(encoder.encode());
                    if result == SatResult::Satisfiable {
                        continue;
                    }

                    found += 1;

                    let graph = encoder.get_graph();

                    // Save the problem and node count.
                    results.push((problem.clone(), graph.graph.node_count()));

                    if let Some(path_dir) = matches_find.value_of("output_svg") {
                        let dot = graph.graph.get_dot();
                        create_dir_all(path_dir).unwrap();
                        let mut path_buf = PathBuf::from(path_dir);
                        let file_name = format!(
                            "{}; n={}; G={}.svg",
                            problem.to_string(),
                            graph.graph.node_count(),
                            graph_index
                        );
                        path_buf.push(file_name);
                        let path = path_buf.as_path().to_str().unwrap();
                        save_as_svg(path, &dot).expect("Failed to save graph as svg.");
                    }

                    if !matches_find.is_present("all_graphs") {
                        break 'encoder_loop;
                    }
                }
                if found > 0 && !matches_find.is_present("all_graph_sizes") {
                    break 'graph_size_loop;
                }
            }

            if results.is_empty() {
                results.push((problem.clone(), 0));
            }

            results
        })
        .collect();
    let time_sat = now.elapsed().as_secs_f32();

    let (nonproven_results, proven_results): (_, Vec<_>) =
        results.into_iter().partition(|(_, n)| *n == 0);

    for (problem, graph_node_count) in &proven_results {
        println!("{}: {}", graph_node_count, problem.to_string());
    }

    if let Some(path) = matches_find.value_of("write_nonproven_result") {
        let f = File::create(path).expect("Unable to create file");
        let mut f = BufWriter::new(f);
        for (p, n) in &nonproven_results {
            f.write_all(format!("{}: {}\n", n, p.to_string()).as_bytes())
                .expect("Unable to write data");
        }
    }

    if matches_find.is_present("print_stats") {
        let new_uniques_len = if matches_find.is_present("all") {
            // This is needed to show the real unique result problem count.
            proven_results.iter().unique_by(|(p, _)| p).count()
        } else {
            proven_results.len()
        };
        eprintln!(
            "Problems were generated/fetched in {} s",
            time_problems,
        );
        eprintln!(
            "Multigraphs were generated/fetched in {} s",
            time_graphs,
        );
        eprintln!(
            "SAT instances were solved in {} s",
            time_sat,
        );
        eprintln!(
            "Total time {} s",
            time_problems + time_graphs + time_sat,
        );

        eprintln!(
            "Found new lower bounds for {}/{} problems",
            new_uniques_len,
            nonproven_results.len() + new_uniques_len
        );

        let sizes = proven_results
            .iter()
            .map(|(_, n)| *n)
            .unique()
            .sorted()
            .collect_vec();

        for n in sizes {
            let count = proven_results.iter().filter(|(_, size)| n == *size).count();
            eprintln!("n = {:2}; count = {:5}", n, count);
        }
    }

    Ok(())
}
