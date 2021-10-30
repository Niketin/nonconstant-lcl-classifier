mod graph_utils;
mod lcl_problem;
mod sat_encoding;
mod sat_solver;

pub use graph_utils::{save_as_svg, BiregularGraph, DotFormat, UndirectedGraph};
pub use lcl_problem::configurations::Configurations;
pub use lcl_problem::LclProblem;
pub use sat_encoding::SatEncoder;
pub use sat_solver::{SatResult, SatSolver};


#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_lcl_on_n4_graphs_unsatisfiable() -> Result<(), Box<dyn std::error::Error>> {
        let n = 4;
        let a = "S S";
        let p = "K K";
        let lcl_problem = LclProblem::new(a, p)?;
        let deg_a = lcl_problem.active.get_labels_per_configuration();
        let deg_p = lcl_problem.passive.get_labels_per_configuration();

        let graphs = BiregularGraph::generate_simple(n, deg_a, deg_p);

        assert!(!graphs.is_empty());

        graphs.into_iter().for_each(|graph| {
            let sat_encoder = SatEncoder::new(lcl_problem.clone(), graph);
            let clauses = sat_encoder.encode();
            let result = SatSolver::solve(&clauses);
            assert_eq!(result, SatResult::Unsatisfiable);
        });

        Ok(())
    }

    #[test]
    fn test_lcl_on_n4_graphs_satisfiable() -> Result<(), Box<dyn std::error::Error>> {
        let n = 5;

        let a = "M U U\nP P P";
        let p = "M M\nP U\nU U";
        let lcl_problem = LclProblem::new(a, p)?;
        let deg_a = lcl_problem.active.get_labels_per_configuration();
        let deg_p = lcl_problem.passive.get_labels_per_configuration();

        let graphs = BiregularGraph::generate_multigraph(n, deg_a, deg_p);

        assert!(!graphs.is_empty());
        graphs.into_iter().for_each(|graph| {
            let sat_encoder = SatEncoder::new(lcl_problem.clone(), graph);
            let clauses = sat_encoder.encode();
            let result = SatSolver::solve(&clauses);
            assert_eq!(result, SatResult::Satisfiable);
        });

        Ok(())
    }

    #[test]
    fn test_lcl_on_n10_graphs_unsatisfiable() -> Result<(), Box<dyn std::error::Error>> {
        let n = 10;

        let a = "M U U\nP P P";
        let p = "M M\nP U\nU U";
        let lcl_problem = LclProblem::new(a, p)?;
        let deg_a = lcl_problem.active.get_labels_per_configuration();
        let deg_p = lcl_problem.passive.get_labels_per_configuration();

        let graphs = BiregularGraph::generate_multigraph(n, deg_a, deg_p);

        assert!(!graphs.is_empty());

        let mut results = graphs.into_iter().map(|graph| {
            let sat_encoder = SatEncoder::new(lcl_problem.clone(), graph);
            let clauses = sat_encoder.encode();
            SatSolver::solve(&clauses)
        });

        // At least one result is unsatisfiable.
        assert!(results.any(|result| { result == SatResult::Unsatisfiable }));

        Ok(())
    }

    #[test]
    fn test_satisfiable_on_small_graph() -> Result<(), Box<dyn std::error::Error>> {
        let n = 2;

        let a = "1 2 3";
        let p = "1 2 3";
        let lcl_problem = LclProblem::new(a, p)?;
        let deg_a = lcl_problem.active.get_labels_per_configuration();
        let deg_p = lcl_problem.passive.get_labels_per_configuration();

        let graphs = BiregularGraph::generate_multigraph(n, deg_a, deg_p);

        assert!(!graphs.is_empty());

        let results = graphs.into_iter().map(|graph| {
            let sat_encoder = SatEncoder::new(lcl_problem.clone(), graph);
            let clauses = sat_encoder.encode();
            sat_encoder.print_clauses(&clauses);
            SatSolver::solve(&clauses)
        }).collect_vec();

        assert!(results.iter().all(|result| { *result == SatResult::Satisfiable }));

        Ok(())
    }


}