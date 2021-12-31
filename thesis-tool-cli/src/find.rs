use crate::from_lcl_classifier::fetch_problems;
use crate::from_stdin::from_stdin;
use clap::{value_t_or_exit, values_t, ArgMatches};
use indicatif::{ParallelProgressIterator, ProgressFinish};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use log::info;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::{path::PathBuf, str::FromStr, time::Instant};
use thesis_tool_lib::lcl_problem::{Normalizable, Purgeable};
use thesis_tool_lib::{
    caches::{GraphSqliteCache, LclProblemSqliteCache},
    save_as_svg, BiregularGraph, DotFormat, LclProblem, SatEncoder, SatResult, SatSolver,
};

pub fn find(matches_find: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
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
            let modulo = values_t!(sub_m, "modulo", u16).ok();

            let modulo = modulo.map(|v| (v[0], v[1]));

            let mut problems =
                fetch_problems(db_path, active_degree, passive_degree, label_count, modulo).expect(
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
        ("from_stdin", Some(_)) => {
            let problems =
                from_stdin().expect(format!("Failed to read problems from stdin",).as_str());
            assert!(problems.len() > 0, "No problems were given to stdin",);
            problems
        }
        (_, _) => unreachable!(),
    };
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

    for n in n_lower..=n_upper {
        // Get biregular graphs from cache or generate them.
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
        .flat_map(|problem| {
            let mut results = vec![];

            'graph_size_loop: for graphs_n in &graphs {
                let now = Instant::now();

                // Create SAT encoders.
                let encoders = graphs_n
                    .into_iter()
                    .map(|graph| SatEncoder::new(&problem, graph.clone())); // TODO use immutable reference instead of cloning.

                // Solve SAT problems.
                let mut unsat_result_index = None;
                for encoder in encoders {
                    let result = SatSolver::solve(&encoder.encode());
                    if result == SatResult::Unsatisfiable {
                        unsat_result_index = Some(encoder);
                        break;
                    }
                }

                info!(
                    "Time used for encoding and solving SAT problems is {} s",
                    now.elapsed().as_secs_f32()
                );

                if unsat_result_index.is_some() {
                    let encoder = unsat_result_index.unwrap();
                    let graph = encoder.get_graph();
                    let dot = graph.graph.get_dot();

                    results.push((problem.clone(), graph.graph.node_count()));

                    if let Some(path) = matches_find.value_of("output_svg") {
                        // TODO save all results as svg, not just the last. Currently the latest svg overrides previous svgs.
                        save_as_svg(path, &dot).expect("Failed to save graph as svg.");
                    }
                    if !matches_find.is_present("all") {
                        break 'graph_size_loop;
                    }
                }
            }

            if results.is_empty() {
                results.push((problem.clone(), 0));
            }

            results
        })
        .collect();

    let (old_results, new_results): (_, Vec<_>) = results.into_iter().partition(|(_, n)| *n == 0);

    for (problem, graph_node_count) in &new_results {
        println!("{}: {}", graph_node_count, problem.to_string());
    }

    if matches_find.is_present("print_stats") {
        let new_uniques_len = if matches_find.is_present("all") {
            // This is needed to show the real unique result problem count.
            new_results.iter().unique_by(|(p, _)| p).count()
        } else {
            new_results.len()
        };
        eprintln!(
            "Found new lower bounds for {}/{} problems",
            new_uniques_len,
            old_results.len() + new_uniques_len
        );

        let sizes = new_results
            .iter()
            .map(|(_, n)| *n)
            .unique()
            .sorted()
            .collect_vec();

        for n in sizes {
            let count = new_results.iter().filter(|(_, size)| n == *size).count();
            eprintln!("n = {:2}; count = {:5}", n, count);
        }
    }

    if let Some(path) = matches_find.value_of("write_old_result") {
        let f = File::create(path).expect("Unable to create file");
        let mut f = BufWriter::new(f);
        for (p, n) in &old_results {
            f.write_all(format!("{}: {}\n", n, p.to_string()).as_bytes())
                .expect("Unable to write data");
        }
    }

    Ok(())
}
