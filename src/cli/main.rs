mod app;
mod find;
mod generate;

use app::build_cli;
use find::find;
use generate::generate;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let matches = build_cli().get_matches();

    if let Some(matches_find) = matches.subcommand_matches("find") {
        find(matches_find)?
    }

    if let Some(matches_generate) = matches.subcommand_matches("gen") {
        generate(matches_generate)?
    }

    Ok(())
}
