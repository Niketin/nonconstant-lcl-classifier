use std::process::Command;
use graph6::string_to_adjacency_matrix;
use petgraph::{Graph, Undirected, dot::{Config, Dot}};

const N: usize = 4;

fn main() {
    let mut command = Command::new("geng");
    let graphs = command.arg("-bc").arg(N.to_string());

    let out = graphs.output().expect("msg");
    let lines = String::from_utf8(out.stdout).expect("Not in utf8 format");
    /*println!(
        "{}",
        &lines
    );*/

    //println!("{}", &lines.lines().count());

    let lines_it = lines.lines();
    for line in lines_it {

        //let graph = lines_it.next().expect("List of graphs is empty");
    
        let adjacency_matrix = string_to_adjacency_matrix(line);
    
        let edges = adjacency_matrix_to_edge_list(adjacency_matrix);
    
        let graph: Graph<usize, (), Undirected, usize> = petgraph::graph::UnGraph::from_edges(&edges);
    
        println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel, Config::NodeIndexLabel]));   
    }

}

fn adjacency_matrix_to_edge_list((adjacency_matrix, size): (Vec<f32>, usize)) -> Vec<(usize, usize)> {
    let mut result: Vec<(usize, usize)> = Vec::new();

    for row in 0..size {
        for col in row..size {
            let value = adjacency_matrix.get(row * size + col).unwrap();
            if value.to_ne_bytes() == 1.0f32.to_ne_bytes() {
                result.push((col, row));
            }
        }
    }

    return result;
}
