mod app;
mod create_cache;
mod find;
mod generate;

use app::build_cli;
use create_cache::create_cache;
use find::find;
use generate::generate;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let matches = build_cli().get_matches();

    match matches.subcommand() {
        ("find", Some(sub_m)) => find(sub_m)?,
        ("gen", Some(sub_m)) => generate(sub_m)?,
        ("create_cache", Some(sub_m)) => create_cache(sub_m)?,
        (_, _) => unreachable!(),
    }

    Ok(())
}
