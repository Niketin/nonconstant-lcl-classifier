use picorust::picosat;

use crate::sat_encoder::Clauses;

/// Enumerator for SAT solver's result.
#[derive(Debug, PartialEq)]
pub enum SatResult {
    Satisfiable,
    Unsatisfiable,
}

/// SAT problem solver.
///
/// The solution from the solver is either [`SatResult::Satisfiable`] or [`SatResult::Unsatisfiable`].
///
/// More about SAT [here](https://en.wikipedia.org/wiki/Boolean_satisfiability_problem).
pub struct SatSolver {}

impl SatSolver {
    /// Solves SAT problem using PicoSAT.
    ///
    /// Returns enumerator [`SatResult`] stating the solver's result.
    pub fn solve(clauses: &Clauses) -> SatResult {
        let mut psat = picosat::init();

        for clause in clauses.iter() {
            for var in clause.iter() {
                picosat::add(&mut psat, *var);
            }
            picosat::add(&mut psat, 0);
        }

        let result = picosat::sat(&mut psat, -1);

        picosat::reset(&mut psat);

        return match result {
            10 => SatResult::Satisfiable,
            20 => SatResult::Unsatisfiable,
            _ => unimplemented!("Unknown result"),
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::{SatResult, SatSolver};

    #[test]
    fn test_solver_returns_satisfiable() {
        // Simple CNF satisfiability problem that is satisfiable.
        let clauses = vec![vec![1, -2, 3, 4]];
        let result = SatSolver::solve(&clauses);
        assert_eq!(result, SatResult::Satisfiable);
    }

    #[test]
    fn test_solver_returns_unsatisfiable() {
        // Simple CNF satisfiability problem that is unsatisfiable.
        let clauses = vec![vec![1], vec![-1]];
        let result = SatSolver::solve(&clauses);
        assert_eq!(result, SatResult::Unsatisfiable);
    }
}
