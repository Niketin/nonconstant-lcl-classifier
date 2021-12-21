use clap::value_t_or_exit;
use clap::ArgMatches;
use std::path::PathBuf;
use std::str::FromStr;
use thesis_tool_lib::caches::graph::multigraph_cache::GraphSqliteHandler;
use thesis_tool_lib::caches::lcl_problem::lcl_problem_cache::LclProblemSqliteHandler;
use thesis_tool_lib::caches::lcl_problem::powerset_cache::PowersetSqliteHandler;
use thesis_tool_lib::BiregularGraph;
use thesis_tool_lib::LclProblem;

pub fn generate(matches_generate: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(matches_graphs) = matches_generate.subcommand_matches("graphs") {
        generate_graphs(matches_graphs)?
    }
    if let Some(matches_problems) = matches_generate.subcommand_matches("problems") {
        generate_problems(matches_problems)?
    }
    Ok(())
}

fn generate_problems(matches_problems: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let active_degree = value_t_or_exit!(matches_problems, "active_degree", usize);
    let passive_degree = value_t_or_exit!(matches_problems, "passive_degree", usize);
    let label_count = value_t_or_exit!(matches_problems, "label_count", usize);
    let sqlite_cache_path = matches_problems.value_of("sqlite_cache");

    let mut problem_cache = if sqlite_cache_path.is_some() {
        Some(LclProblemSqliteHandler::new(
            PathBuf::from_str(sqlite_cache_path.unwrap())
                .expect("Database at the given path does not exist"),
        ))
    } else {
        None
    };

    let mut powerset_cache = if sqlite_cache_path.is_some() {
        Some(PowersetSqliteHandler::new(
            PathBuf::from_str(sqlite_cache_path.unwrap())
                .expect("Database at the given path does not exist"),
        ))
    } else {
        None
    };

    let problems =
        LclProblem::get_or_generate_normalized::<LclProblemSqliteHandler, PowersetSqliteHandler>(
            active_degree,
            passive_degree,
            label_count as u8,
            problem_cache.as_mut(),
            powerset_cache.as_mut(),
        );

    for problem in problems {
        println!("{}", problem.to_string());
    }
    Ok(())
}

fn generate_graphs(matches_graphs: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let min_nodes = value_t_or_exit!(matches_graphs, "min_nodes", usize);
    let max_nodes = value_t_or_exit!(matches_graphs, "max_nodes", usize);
    let active_degree = value_t_or_exit!(matches_graphs, "active_degree", usize);
    let passive_degree = value_t_or_exit!(matches_graphs, "passive_degree", usize);
    let sqlite_cache_path = matches_graphs.value_of("sqlite_cache");

    let mut cache = if sqlite_cache_path.is_some() {
        Some(GraphSqliteHandler::new(
            PathBuf::from_str(sqlite_cache_path.unwrap())
                .expect("Database at the given path does not exist"),
        ))
    } else {
        None
    };

    let mut sum = 0usize;
    for n in min_nodes..=max_nodes {
        let graphs =
            BiregularGraph::get_or_generate(n, active_degree, passive_degree, cache.as_mut());
        sum += graphs.len();
    }
    eprintln!("Generated {} multigraphs!", sum);

    Ok(())
}
