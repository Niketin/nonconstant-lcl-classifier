mod biregular_graph;
mod dot_format;

use graph6::string_to_adjacency_matrix;
use itertools::Itertools;

use petgraph::{graph::NodeIndex, Graph, Undirected};
use std::io::prelude::*;
use std::{fs::File, process::Command, process::Stdio};

pub use biregular_graph::BiregularGraph;
pub use dot_format::DotFormat;

fn generate_bipartite_graphs_graph8(graph_size: usize) -> String {
    // Use geng and assume it exists in the system.
    let mut command = Command::new("geng");

    // Flag -b gives us bipartite graphs and -c gives us connected graphs.
    let graphs = command.arg("-bc").arg(graph_size.to_string());

    let out = graphs.output().expect("msg");
    String::from_utf8(out.stdout).expect("Not in utf8 format")
}

fn graph6_to_petgraph(graph: &str) -> Graph<u32, (), Undirected, u32> {
    let adjacency_matrix = string_to_adjacency_matrix(graph);
    let edges = adjacency_matrix_to_edge_list(adjacency_matrix);
    petgraph::graph::UnGraph::from_edges(&edges)
}

/// Writes dot graph into svg file.
pub fn save_as_svg(path: &str, dot: &str) -> Result<(), Box<dyn std::error::Error>> {
    let process = Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("couldn't spawn wc");

    process
        .stdin
        .unwrap()
        .write_all(dot.as_bytes())
        .expect("couldn't write to dot stdin:");

    let mut s = String::new();
    process
        .stdout
        .unwrap()
        .read_to_string(&mut s)
        .expect("couldn't read dot stdout:");

    let mut file = File::create(path)?;
    file.write_all(s.as_bytes())?;

    Ok(())
}

/// Transforms adjacency matrix into list of edges.
fn adjacency_matrix_to_edge_list((adjacency_matrix, size): (Vec<f32>, usize)) -> Vec<(u32, u32)> {
    let mut result: Vec<(u32, u32)> = Vec::new();

    // Iterate upper triangle of the adjacency matrix
    for row in 0..size {
        for col in row..size {
            let value = adjacency_matrix.get(row * size + col).unwrap();
            if value.to_ne_bytes() == 1.0f32.to_ne_bytes() {
                result.push((col as u32, row as u32));
            }
        }
    }

    return result;
}

fn generate_bipartite_graphs_with_partition_sizes_and_degree_bounds_graph8(
    n1: usize,
    n2: usize,
    d1_low: usize,
    d2_low: usize,
    d1_high: usize,
    d2_high: usize,
) -> String {
    // Use geng and assume it exists in the system.
    let mut command = Command::new("genbg");

    let parameter_degree_lower_bound = format!("-d{}:{}", d1_low, d2_low);
    let parameter_degree_upper_bound = format!("-D{}:{}", d1_high, d2_high);

    // Flag -c gives us connected graphs.
    let graphs = command
        .arg("-c")
        .arg(parameter_degree_lower_bound)
        .arg(parameter_degree_upper_bound)
        .arg(n1.to_string())
        .arg(n2.to_string());

    let out = graphs.output().expect("msg");
    String::from_utf8(out.stdout).expect("Not in utf8 format")
}

fn generate_biregular_graphs_with_partition_sizes_graph8(
    n1: usize,
    n2: usize,
    d1: usize,
    d2: usize,
) -> String {
    generate_bipartite_graphs_with_partition_sizes_and_degree_bounds_graph8(n1, n2, d1, d2, d1, d2)
}

fn generate_biregular_graphs_with_total_size_graph8(
    n: usize,
    d1: usize,
    d2: usize,
) -> Vec<((usize, usize), String)> {
    let mut graphs = Vec::new();
    for (n1, n2) in biregular_partition_sizes(n, d1, d2) {
        graphs.push((
            (n1, n2),
            generate_biregular_graphs_with_partition_sizes_graph8(n1, n2, d1, d2),
        ));
    }
    graphs
}

fn b_sums(n: usize) -> Vec<(usize, usize)> {
    (1..=(n / 2)).map(|i| (i, n - i)).collect_vec()
}

fn biregular_partition_sizes(n: usize, d1: usize, d2: usize) -> Vec<(usize, usize)> {
    b_sums(n)
        .iter()
        .filter_map(|(n1, n2)| {
            if d1 * n1 == d2 * n2 {
                return Some((*n1, *n2));
            } else if d1 * n2 == d2 * n1 {
                return Some((*n2, *n1));
            }
            None
        })
        .collect_vec()
}

fn get_partitions(
    graph: &Graph<u32, (), Undirected, u32>,
    n1: usize,
    n2: usize,
) -> (Vec<NodeIndex<u32>>, Vec<NodeIndex<u32>>) {
    assert_eq!(graph.node_count(), n1 + n2);

    let node_indices_a: Vec<NodeIndex<u32>> = graph
        .node_indices()
        .filter(|i| i.index() < n1)
        .collect_vec();
    let node_indices_p: Vec<NodeIndex<u32>> = graph
        .node_indices()
        .filter(|i| i.index() >= n1)
        .collect_vec();

    (node_indices_a, node_indices_p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generating_biregular_graphs() {
        assert_eq!(count_biregular_graphs(5, 3, 2), 1);
        assert_eq!(count_biregular_graphs(5, 2, 3), 1);

        assert_eq!(count_biregular_graphs(7, 2, 3), 0);
        assert_eq!(count_biregular_graphs(7, 3, 2), 0);

        assert_eq!(count_biregular_graphs(8, 5, 3), 1);
        assert_eq!(count_biregular_graphs(8, 3, 5), 1);

        assert_eq!(count_biregular_graphs(8, 3, 3), 1);
    }

    fn count_biregular_graphs(n: usize, a: usize, b: usize) -> usize {
        BiregularGraph::generate(n, a, b).len()
    }

    #[test]
    fn test_biregular_graph_partitions_have_correct_degrees() {
        let graphs = BiregularGraph::generate(5, 3, 2);

        for graph in graphs {
            assert_eq!(graph.degree_a, 3);
            assert_eq!(graph.degree_b, 2);
            for node in graph.partition_a {
                assert_eq!(graph.graph.neighbors(node).count(), 3)
            }

            for node in graph.partition_b {
                assert_eq!(graph.graph.neighbors(node).count(), 2)
            }
        }
    }

    #[test]
    fn test_b_sums() {
        assert_eq!(b_sums(3), vec![(1, 2)]);
        assert_eq!(b_sums(4), vec![(1, 3), (2, 2)]);
        assert_eq!(b_sums(5), vec![(1, 4), (2, 3)]);
    }

    #[test]
    fn test_biregular_partition_sizes() {
        assert_eq!(biregular_partition_sizes(5, 2, 3).len(), 1);
        assert_eq!(biregular_partition_sizes(5, 3, 2).len(), 1);
    }
}
