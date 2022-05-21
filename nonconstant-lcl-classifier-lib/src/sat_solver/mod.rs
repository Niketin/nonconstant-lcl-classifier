use crate::sat_encoder::Clauses;
use kissat_rs;
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
    /// Solves SAT problem using Kissat SAT solver.
    ///
    /// Returns enumerator [`SatResult`] stating the solver's result.
    pub fn solve(clauses: Clauses) -> SatResult {
        let unsat_result = kissat_rs::Solver::decide_formula(clauses).unwrap();
        match unsat_result {
            true => SatResult::Satisfiable,
            false => SatResult::Unsatisfiable,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{SatResult, SatSolver};

    #[test]
    fn test_solver_returns_satisfiable() {
        // Simple CNF satisfiability problem that is satisfiable.
        let clauses = vec![vec![1, -2, 3, 4]];
        let result = SatSolver::solve(clauses);
        assert_eq!(result, SatResult::Satisfiable);
    }

    #[test]
    fn test_solver_returns_unsatisfiable() {
        // Simple CNF satisfiability problem that is unsatisfiable.
        let clauses = vec![vec![1], vec![-1]];
        let result = SatSolver::solve(clauses);
        assert_eq!(result, SatResult::Unsatisfiable);
    }
}
