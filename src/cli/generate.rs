use clap::ArgMatches;
use clap::{value_t_or_exit};
use thesis_tool_lib::BiregularGraph;


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
    todo!()
}

fn generate_graphs(matches_graphs: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let min_nodes = value_t_or_exit!(matches_graphs, "min_nodes", usize);
    let max_nodes = value_t_or_exit!(matches_graphs, "max_nodes", usize);
    let active_degree = value_t_or_exit!(matches_graphs, "active_degree", usize);
    let passive_degree = value_t_or_exit!(matches_graphs, "passive_degree", usize);

    let mut sum = 0usize;
    for n in min_nodes..=max_nodes {
        // TODO do this in parallel by modifying underlying functions as you see fit.
        // TODO Probably this way:
        // TODO Thread 1: generate nth part of simple graphs (and cache) -> extend to multigraphs (and cache) -> return them
        // TODO Thread 2: generate nth part of simple graphs (and cache) -> extend to multigraphs (and cache) -> return them
        // TODO ...
        // TODO Thread n: generate nth part of simple graphs (and cache) -> extend to multigraphs (and cache) -> return them
        let graphs = BiregularGraph::get_or_generate_multigraphs_parallel(n, active_degree, passive_degree);
        //let graphs = BiregularGraph::generate_multigraphs(n, active_degree, passive_degree);
        sum += graphs.len();
        println!();
    }
    eprintln!("Generated {} multigraphs!", sum);

    Ok(())
}
