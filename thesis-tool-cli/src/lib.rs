pub mod app;
pub mod create_cache;
pub mod find;
pub mod generate;
pub mod utils;

use crate::create_cache::create_cache;
use crate::find::find;
use crate::generate::generate;
use std::error::Error;

pub fn run_subcommand(matches: clap::ArgMatches) -> Result<(), Box<dyn Error>> {
    Ok(match matches.subcommand() {
        ("find", Some(sub_m)) => find(sub_m)?,
        ("gen", Some(sub_m)) => generate(sub_m)?,
        ("create_cache", Some(sub_m)) => create_cache(sub_m)?,
        (_, _) => unreachable!(),
    })
}
