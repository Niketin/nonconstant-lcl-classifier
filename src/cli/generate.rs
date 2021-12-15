use std::path::PathBuf;
use std::str::FromStr;

use clap::value_t_or_exit;
use clap::ArgMatches;
use thesis_tool_lib::graph_caches::multigraph_cache::SqliteCacheHandler;
use thesis_tool_lib::{BiregularGraph};

pub fn generate(matches_generate: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(matches_graphs) = matches_generate.subcommand_matches("graphs") {
        generate_graphs(matches_graphs)?
    }
    if let Some(matches_problems) = matches_generate.subcommand_matches("problems") {
        generate_problems(matches_problems)?
    }
    Ok(())
}

fn generate_problems(_matches_problems: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}

fn generate_graphs(matches_graphs: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let min_nodes = value_t_or_exit!(matches_graphs, "min_nodes", usize);
    let max_nodes = value_t_or_exit!(matches_graphs, "max_nodes", usize);
    let active_degree = value_t_or_exit!(matches_graphs, "active_degree", usize);
    let passive_degree = value_t_or_exit!(matches_graphs, "passive_degree", usize);
    let sqlite_cache_path = matches_graphs.value_of("sqlite_cache");

    let mut cache = if sqlite_cache_path.is_some() {
        Some(SqliteCacheHandler::new(
            PathBuf::from_str(sqlite_cache_path.unwrap())
                .expect("Database at the given path does not exist"),
        ))
    } else {
        None
    };

    let mut sum = 0usize;
    for n in min_nodes..=max_nodes {
        let graphs = BiregularGraph::get_or_generate_multigraphs_parallel(
            n,
            active_degree,
            passive_degree,
            cache.as_mut(),
        );
        sum += graphs.len();
    }
    eprintln!("Generated {} multigraphs!", sum);

    Ok(())
}
