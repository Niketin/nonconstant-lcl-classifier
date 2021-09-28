use clap::{value_t_or_exit, App, Arg};
use itertools::Itertools;
use std::io;
use std::io::stdout;
use std::io::Write;
use thesis_tool_lib::*;

macro_rules! print_flush {
    ( $($t:tt)* ) => {
        {
            let mut h = stdout();
            write!(h, $($t)* ).unwrap();
            h.flush().unwrap();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Thesis tool")
        .about("This tool can be used to find negative proofs of LCL-problems solvability on the Port Numbering model.")
        .arg(Arg::with_name("n")
            .help("Sets the number of vertices in the graphs.")
            .index(1)
            .required(true)
        )
        .arg(Arg::with_name("active_configurations")
            .help("Sets the active configurations of the LCL-problem.")
            .index(2)
            .required(true)
        )
        .arg(Arg::with_name("passive_configurations")
            .help("Sets the passive configurations of the LCL-problem.")
            .index(3)
            .required(true)
        )
        .get_matches();

    let n = value_t_or_exit!(matches, "n", usize);
    let a = matches
        .value_of("active_configurations")
        .expect("Parsing parameter 'a' failed.");
    let p = matches
        .value_of("passive_configurations")
        .expect("Parsing parameter 'p' failed.");

    let lcl_problem = LclProblem::new(a, p).expect("Creating LclProblem failed.");
    print_flush!("Generating biregular nonisomorphic graphs (n={})...", n);
    let graphs = generate_biregular_graphs(
        n,
        lcl_problem.active.get_labels_per_configuration(),
        lcl_problem.passive.get_labels_per_configuration(),
    );
    println!(" {} graphs generated.", graphs.len());

    let encodings = graphs
        .into_iter()
        .enumerate()
        .map(|(i, graph)| {
            print_flush!("Encoding graph into SAT problem {}... ", i);
            let sat_encoder = SatEncoder::new(lcl_problem.clone(), graph);
            let result = sat_encoder.encode();
            println!("done!");
            result
        })
        .collect_vec();

    encodings.into_iter().enumerate().for_each(|(i, encoding)| {
        print_flush!("solving SAT problem {}... ", i + 1);
        let result = SatSolver::solve(encoding);
        println!("{:?}", result);
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
