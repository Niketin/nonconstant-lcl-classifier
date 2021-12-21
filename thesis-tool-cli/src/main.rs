use std::error::Error;

use thesis_tool_cli_lib::app::build_cli;
use thesis_tool_cli_lib::run_subcommand;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let matches = build_cli().get_matches();

    run_subcommand(matches)?;

    Ok(())
}

#[cfg(test)]
mod cli_tests {
    use std::error::Error;
    use thesis_tool_cli_lib::utils::*;

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
        create_graphs(path, 1, 16, 3, 3)?;
        create_graphs(path, 1, 16, 3, 3)?;
        Ok(())
    }

    #[test]
    fn test_generate_problems() -> Result<(), Box<dyn Error>> {
        let path = "/tmp/tool_test_cache_2.db";
        create_cache(path)?;
        create_problems(path, 2, 2, 2)?;
        create_problems(path, 2, 2, 2)?;
        Ok(())
    }
}
