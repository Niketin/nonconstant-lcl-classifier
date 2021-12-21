use std::error::Error;

use crate::{app::build_cli, run_subcommand};

pub fn execute_app(args: &str) -> Result<(), Box<dyn Error>> {
    let matches = build_cli()
        .setting(clap::AppSettings::NoBinaryName)
        .get_matches_from(args.split_ascii_whitespace());
    run_subcommand(matches)?;
    Ok(())
}

pub fn create_cache(path: &str) -> Result<(), Box<dyn Error>> {
    execute_app(format!("create_cache {}", path).as_str())?;
    Ok(())
}

pub fn create_graphs(cache_path: &str, n_low: usize, n_high: usize, deg_a: usize, deg_p: usize) -> Result<(), Box<dyn Error>> {
    execute_app(format!("gen graphs -c {} {} {} {} {}", cache_path, n_low, n_high, deg_a, deg_p).as_str())?;
    Ok(())
}

pub fn create_problems(path: &str, deg_a: usize, deg_p: usize, labels: usize) -> Result<(), Box<dyn Error>> {
    execute_app(format!("gen problems -c {} {} {} {}", path, deg_a, deg_p, labels).as_str())?;
    Ok(())
}
