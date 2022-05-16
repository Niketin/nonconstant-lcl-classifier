use clap::ArgMatches;
use nonconstant_lcl_classifier_lib::caches::create_sqlite_cache;

pub fn create_cache(matches_graphs: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let sqlite_cache_path = matches_graphs.value_of("sqlite_cache");
    eprintln!("Trying to create a new SQLite database for caching...");

    create_sqlite_cache(sqlite_cache_path.unwrap())?;
    eprintln!("Created!");

    Ok(())
}
