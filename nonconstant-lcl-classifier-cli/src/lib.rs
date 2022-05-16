pub mod app;
pub mod create_cache;
pub mod find;
pub mod from_lcl_classifier;
pub mod generate;
pub mod utils;
pub mod from_stdin;

use crate::create_cache::create_cache;
use crate::find::find;
use crate::generate::generate;
use crate::from_lcl_classifier::fetch_and_print_problems;
use std::error::Error;

pub fn run_subcommand(matches: clap::ArgMatches) -> Result<(), Box<dyn Error>> {
    Ok(match matches.subcommand() {
        ("find", Some(sub_m)) => find(sub_m)?,
        ("gen", Some(sub_m)) => generate(sub_m)?,
        ("create_cache", Some(sub_m)) => create_cache(sub_m)?,
        ("fetch_problems", Some(sub_m)) => fetch_and_print_problems(sub_m)?,
        (_, _) => unreachable!(),
    })
}
