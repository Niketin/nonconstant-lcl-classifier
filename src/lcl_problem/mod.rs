mod configurations;

use configurations::Configurations;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LclProblem {
    pub active: Configurations,
    pub passive: Configurations,
    pub symbol_map: HashMap<String, u8>,
}

impl LclProblem {
    pub fn new(a: &str, p: &str) -> Result<LclProblem, Box<dyn std::error::Error>> {
        let mut symbol_map: HashMap<String, u8> = HashMap::new();
        Ok(LclProblem {
            active: Configurations::from_str(a, &mut symbol_map)?,
            passive: Configurations::from_str(p, &mut symbol_map)?,
            symbol_map,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lcl_problem() {
        const A: &'static str = "M U U\nP P P";
        const P: &'static str = "M M\nP U\nU U";

        let problem = LclProblem::new(A, P);

        assert_eq!(problem.is_ok(), true);
    }
}
