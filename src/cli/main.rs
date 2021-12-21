mod app;
mod create_cache;
mod find;
mod generate;

use std::error::Error;

use app::build_cli;
use create_cache::create_cache;
use find::find;
use generate::generate;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let matches = build_cli().get_matches();

    run_subcommand(matches)?;

    Ok(())
}

fn run_subcommand(matches: clap::ArgMatches) -> Result<(), Box<dyn Error>> {
    Ok(match matches.subcommand() {
        ("find", Some(sub_m)) => find(sub_m)?,
        ("gen", Some(sub_m)) => generate(sub_m)?,
        ("create_cache", Some(sub_m)) => create_cache(sub_m)?,
        (_, _) => unreachable!(),
    })
}

#[cfg(test)]
mod cli_tests {
    use std::error::Error;

    use crate::{app::build_cli, run_subcommand};

    fn execute_app(args: &str) -> Result<(), Box<dyn Error>> {
        let matches = build_cli()
            .setting(clap::AppSettings::NoBinaryName)
            .get_matches_from(args.split_ascii_whitespace());
        run_subcommand(matches)?;
        Ok(())
    }

    fn create_cache(path: &str) -> Result<(), Box<dyn Error>> {
        execute_app(format!("create_cache {}", path).as_str())?;
        Ok(())
    }

    fn create_graphs(path: &str) -> Result<(), Box<dyn Error>> {
        execute_app(format!("gen graphs -c {} 1 16 3 3", path).as_str())?;
        Ok(())
    }

    fn create_problems(path: &str) -> Result<(), Box<dyn Error>> {
        execute_app(format!("gen problems -c {} 2 2 2", path).as_str())?;
        Ok(())
    }

    #[test]
    fn test_create_cache() -> Result<(), Box<dyn Error>> {
        let path = "/tmp/tool_test_cache_0.db";
        create_cache(path)?;
        Ok(())
    }

    #[test]
    fn test_generate_graphs() -> Result<(), Box<dyn Error>> {
        let path = "/tmp/tool_test_cache_1.db";
        create_cache(path)?;
        create_graphs(path)?;
        create_graphs(path)?;
        Ok(())
    }

    #[test]
    fn test_generate_problems() -> Result<(), Box<dyn Error>> {
        let path = "/tmp/tool_test_cache_2.db";
        create_cache(path)?;
        create_problems(path)?;
        create_problems(path)?;
        Ok(())
        //TODO      Running `target/release/thesis_tool_cli gen problems 2 2 2 -c /tmp/tool_test_cache_2.db`
        //TODO [2021-12-15T23:08:30Z ERROR thesis_tool_lib::lcl_problem] Failed writing problems to the cache!

    }
}
