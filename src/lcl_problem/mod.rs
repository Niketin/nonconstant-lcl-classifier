mod encoding;

use encoding::Encoding;
use std::collections::HashMap;

#[derive(Debug)]
pub struct LclProblem {
    pub a: Encoding,
    pub p: Encoding,
    pub symbol_map: HashMap<String, u8>,
}

impl LclProblem {
    pub fn new(a: &str, p: &str) -> Result<LclProblem, Box<dyn std::error::Error>> {
        let mut symbol_map: HashMap<String, u8> = HashMap::new();
        Ok(LclProblem {
            a: Encoding::from_str(a, &mut symbol_map)?,
            p: Encoding::from_str(p, &mut symbol_map)?,
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
