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
