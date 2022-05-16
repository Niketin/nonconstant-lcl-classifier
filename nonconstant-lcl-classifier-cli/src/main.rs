use std::error::Error;

use nonconstant_lcl_classifier_cli::app::build_cli;
use nonconstant_lcl_classifier_cli::run_subcommand;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let matches = build_cli().get_matches();

    run_subcommand(matches)?;

    Ok(())
}

#[cfg(test)]
mod cli_tests {
    use std::error::Error;
    use nonconstant_lcl_classifier_cli::utils::*;

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
        create_graphs_cached(path, 1, 16, 3, 3)?;
        create_graphs_cached(path, 1, 16, 3, 3)?;
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
