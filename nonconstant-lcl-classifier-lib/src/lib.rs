pub mod caches;
mod graph_utils;
pub mod lcl_problem;
pub mod sat_encoder;
pub mod sat_solver;

pub use graph_utils::{save_as_svg, BiregularGraph, DotFormat, UndirectedGraph};
pub use lcl_problem::configurations::Configurations;
pub use lcl_problem::LclProblem;
pub use sat_encoder::SatEncoder;
pub use sat_solver::{SatResult, SatSolver};
//pub use caches::{GraphCacheParams, GraphSqliteCache};

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_lcl_on_n4_graphs_unsatisfiable() -> Result<(), Box<dyn std::error::Error>> {
        let n = 4;
        let a = "SS";
        let p = "KK";
        let lcl_problem = LclProblem::new(a, p)?;
        let deg_a = lcl_problem.active.get_labels_per_configuration();
        let deg_p = lcl_problem.passive.get_labels_per_configuration();

        let graphs = BiregularGraph::generate(n, deg_a, deg_p);

        assert!(!graphs.is_empty());

        graphs.into_iter().for_each(|graph| {
            let sat_encoder = SatEncoder::new(&lcl_problem, graph);
            let clauses = sat_encoder.encode();
            let result = SatSolver::solve(clauses);
            assert_eq!(result, SatResult::Unsatisfiable);
        });

        Ok(())
    }

    #[test]
    fn test_lcl_on_n4_graphs_satisfiable() -> Result<(), Box<dyn std::error::Error>> {
        let n = 5;

        let a = "MUU PPP";
        let p = "MM PU UU";
        let lcl_problem = LclProblem::new(a, p)?;
        let deg_a = lcl_problem.active.get_labels_per_configuration();
        let deg_p = lcl_problem.passive.get_labels_per_configuration();

        let graphs = BiregularGraph::generate(n, deg_a, deg_p);

        assert!(!graphs.is_empty());
        graphs.into_iter().for_each(|graph| {
            let sat_encoder = SatEncoder::new(&lcl_problem, graph);
            let clauses = sat_encoder.encode();
            let result = SatSolver::solve(clauses);
            assert_eq!(result, SatResult::Satisfiable);
        });

        Ok(())
    }

    #[test]
    fn test_lcl_on_n10_graphs_unsatisfiable() -> Result<(), Box<dyn std::error::Error>> {
        let n_min = 1;
        let n_max = 10;

        let a = "MUU PPP";
        let p = "MM PU UU";
        let lcl_problem = LclProblem::new(a, p)?;
        let deg_a = lcl_problem.active.get_labels_per_configuration();
        let deg_p = lcl_problem.passive.get_labels_per_configuration();

        let graphs_grouped = (n_min..=n_max).map(|n| BiregularGraph::generate(n, deg_a, deg_p));

        let results_grouped = graphs_grouped
            .into_iter()
            .map(|graphs| {
                graphs
                    .into_iter()
                    .map(|graph| {
                        let sat_encoder = SatEncoder::new(&lcl_problem, graph);
                        let clauses = sat_encoder.encode();
                        SatSolver::solve(clauses)
                    })
                    .collect_vec()
            })
            .collect_vec();

        // For n=(1..=9) all results should be satisfiable.
        let (last, rest) = results_grouped.as_slice().split_last().unwrap();
        for results in rest {
            assert!(results.iter().all(|r| *r == SatResult::Satisfiable));
        }

        // For n=10 at least one results should be unsatisfiable.
        assert!(last.iter().any(|r| *r == SatResult::Unsatisfiable));

        Ok(())
    }

    #[test]
    fn test_satisfiable_on_small_graph() -> Result<(), Box<dyn std::error::Error>> {
        let n = 2;

        let a = "123";
        let p = "123";
        let lcl_problem = LclProblem::new(a, p)?;
        let deg_a = lcl_problem.active.get_labels_per_configuration();
        let deg_p = lcl_problem.passive.get_labels_per_configuration();

        let graphs = BiregularGraph::generate(n, deg_a, deg_p);

        assert!(!graphs.is_empty());

        let results = graphs
            .into_iter()
            .map(|graph| {
                let sat_encoder = SatEncoder::new(&lcl_problem, graph);
                let clauses = sat_encoder.encode();
                sat_encoder.print_clauses(&clauses);
                SatSolver::solve(clauses)
            })
            .collect_vec();

        assert!(results
            .iter()
            .all(|result| { *result == SatResult::Satisfiable }));

        Ok(())
    }
}
