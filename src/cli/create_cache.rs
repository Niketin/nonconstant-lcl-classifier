
use clap::ArgMatches;
use thesis_tool_lib::graph_caches::multigraph_cache::create_database;


pub fn create_cache(matches_graphs: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_cache_path = matches_graphs.value_of("sqlite_cache");
    eprintln!("Trying to create a new SQLite database for caching...");

    create_database(sqlite_cache_path.unwrap())?;
    eprintln!("Created!");

    Ok(())
}
