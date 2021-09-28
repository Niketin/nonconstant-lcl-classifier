use std::env;
use thesis_tool_lib::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    // 3 arguments are required.
    assert_eq!(args.len(), 3);

    let n = &args[0].parse::<usize>()?;
    let deg_a = &args[1].parse::<usize>()?;
    let deg_p = &args[2].parse::<usize>()?;

    // Generate graphs.
    let graphs = generate_biregular_graphs(*n, *deg_a, *deg_p);

    // Print each graph in dot format.
    graphs.into_iter().enumerate().for_each(|(i,x)| {
        println!(
            "{}: {:?}, {}: {:?}",
            x.degree_a, x.partition_a, x.degree_b, x.partition_b
        );
        let dot = &x.graph.get_dot();
        println!("{}", dot);

        let path = format!("./graph_{}.svg", i);
        save_as_svg(&path, dot).expect(format!("Saving to path {} did not work", path).as_str());
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcl_on_n4_graphs_unsatisfiable() -> Result<(), Box<dyn std::error::Error>> {
        let n = 4;
        let a = "S S";
        let p = "K K";
        let lcl_problem = LclProblem::new(a, p)?;
        let deg_a = lcl_problem.active.get_labels_per_configuration();
        let deg_p = lcl_problem.passive.get_labels_per_configuration();

        let graphs = generate_biregular_graphs(n, deg_a, deg_p);

        assert!(!graphs.is_empty());

        graphs.into_iter().for_each(|graph| {
            let sat_encoder = SatEncoder::new(lcl_problem.clone(), graph);
            let clauses = sat_encoder.encode();
            let result = SatSolver::solve(clauses);
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

        let graphs = generate_biregular_graphs(n, deg_a, deg_p);

        assert!(!graphs.is_empty());
        graphs.into_iter().for_each(|graph| {
            let sat_encoder = SatEncoder::new(lcl_problem.clone(), graph);
            let clauses = sat_encoder.encode();
            let result = SatSolver::solve(clauses);
            assert_eq!(result, SatResult::Satisfiable);
        });

        Ok(())
    }
}
