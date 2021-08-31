use thesis_tool_lib::*;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    // 2 arguments are required. 
    assert_eq!(args.len(), 2);
    
    // First one for active nodes.
    // Second one for passive nodes.
    let (a, p) = (&args[0], &args[1]);

    // Create LclProblem.
    let problem = LclProblem::new(&a, &p)?;
    println!("{:?}", problem);

    // Generate graphs.
    let graphs = generate_biregular_graphs(9, 2, 3);

    // Print each graph in dot format.
    graphs.into_iter().for_each(|x| {
        println!(
            "{}: {:?}, {}: {:?}",
            x.degree_a, x.partition_a, x.degree_b, x.partition_b
        );
        let dot = &x.graph.get_dot();
        println!("{}", dot);
    });

    Ok(())
}
