mod biregular_graph;

use graph6::string_to_adjacency_matrix;
use petgraph::{
    dot::{Config, Dot},
    graph::NodeIndex,
    visit::VisitMap,
    Graph, Undirected,
};
use std::{collections::HashSet, fs::File, process::Command, process::Stdio};
use std::{fmt::Debug, io::prelude::*};

pub use biregular_graph::BiregularGraph;

/// Generates simple nonisomorphic biregular graphs.
pub fn generate_biregular_graphs(
    graph_size: usize,
    degree_a: usize,
    degree_b: usize,
) -> Vec<BiregularGraph> {
    // Use geng and assume it exists in the system.
    let mut command = Command::new("geng");

    // Flag -b gives us bipartite graphs and -c gives us connected graphs.
    let graphs = command.arg("-bc").arg(graph_size.to_string());

    let out = graphs.output().expect("msg");
    let lines = String::from_utf8(out.stdout).expect("Not in utf8 format");
    let lines_it = lines.lines();

    let mut graphs_petgraph: Vec<BiregularGraph> = Vec::new();

    // Iterate through geng results.
    for line in lines_it {
        // Geng outputs the graphs in Graph6-format.
        // These need to be transformed into adjacency matrix.
        let adjacency_matrix = string_to_adjacency_matrix(line);
        let edges = adjacency_matrix_to_edge_list(adjacency_matrix);
        let graph: Graph<u32, (), Undirected, u32> = petgraph::graph::UnGraph::from_edges(&edges);
        let indices = &graph.node_indices().collect::<Vec<_>>();
        let res: Option<(Vec<NodeIndex>, Vec<NodeIndex>)> = is_bipartite(&graph, indices[0]);
        if let Some((node_indices_a, node_indices_b)) = res {
            let biregular_graph =
                match is_biregular(&graph, &node_indices_a, &node_indices_b, degree_a, degree_b) {
                    None => continue,
                    Some((a, b, c, d)) => BiregularGraph {
                        graph,
                        partition_a: a.clone(),
                        partition_b: b.clone(),
                        degree_a: c,
                        degree_b: d,
                    },
                };
            // Save the graph.
            graphs_petgraph.push(biregular_graph);
        }
    }

    return graphs_petgraph;
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

/// Trait for things that can have a representation in .dot format.
pub trait DotFormat {
    fn get_dot(&self) -> String;
}

impl<N, E> DotFormat for Graph<N, E, Undirected>
where
    E: Debug,
    N: Debug,
{
    fn get_dot(&self) -> String {
        format!(
            "{:?}",
            Dot::with_config(&self, &[Config::EdgeNoLabel, Config::NodeIndexLabel])
        )
    }
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

/// Checks bipartity of a graph and returns the partitions.
fn is_bipartite(
    graph: &Graph<u32, (), Undirected, u32>,
    start: NodeIndex<u32>,
) -> Option<(Vec<NodeIndex<u32>>, Vec<NodeIndex<u32>>)> {
    let mut red: HashSet<NodeIndex<u32>> = HashSet::with_capacity(graph.node_count());
    red.visit(start);
    let mut blue: HashSet<NodeIndex<u32>> = HashSet::with_capacity(graph.node_count());

    let mut stack = ::std::collections::VecDeque::new();
    stack.push_front(start);

    while let Some(node) = stack.pop_front() {
        let is_red = red.contains(&node);
        let is_blue = blue.contains(&node);

        assert!(is_red ^ is_blue);

        for neighbour in graph.neighbors(node) {
            let is_neigbour_red = red.is_visited(&neighbour);
            let is_neigbour_blue = blue.is_visited(&neighbour);

            if (is_red && is_neigbour_red) || (is_blue && is_neigbour_blue) {
                return None; // Not bipartite
            }

            if !is_neigbour_red && !is_neigbour_blue {
                match (is_red, is_blue) {
                    (true, false) => {
                        blue.visit(neighbour);
                    }
                    (false, true) => {
                        red.visit(neighbour);
                    }
                    (_, _) => {
                        panic!("The invariant doesn't hold");
                    }
                }

                stack.push_back(neighbour);
            }
        }
    }
    let red_vec: Vec<NodeIndex<u32>> = red.into_iter().collect();
    let blue_vec: Vec<NodeIndex<u32>> = blue.into_iter().collect();
    Some((red_vec, blue_vec))
}

/// Check if all nodes at node_indices have the specified degree.
///
/// * `graph` - Graph which nodes are checked against the degree criterion.
/// * `node_indices` - Indices of the nodes which will be checked.
/// * `degree` - The degree.
fn all_nodes_with_degree(
    graph: &Graph<u32, (), Undirected, u32>,
    node_indices: &Vec<NodeIndex<u32>>,
    degree: usize,
) -> bool {
    node_indices
        .into_iter()
        .all(|x| &graph.neighbors(*x).count() == &degree)
}

/// Checks if the graph is biregular.
///
/// Graph is biregular if it is bipartite, and
/// nodes in set A have degree degree_a
/// and
/// nodes in set B have degree degree_b
///
/// Bipartity is assumed and not checked.
///
/// * `graph` - Graph which is checked against the biregularity criterion.
/// * `node_indices_a` - Indices of the nodes in partition a.
/// * `node_indices_b` - Indices of the nodes in partition b.
/// * `degree_a` - The assumed degree of nodes in partition a.
/// * `degree_b` - The assumed degree of nodes in partition b.
fn is_biregular<'a>(
    graph: &Graph<u32, (), Undirected, u32>,
    node_indices_a: &'a Vec<NodeIndex<u32>>,
    node_indices_b: &'a Vec<NodeIndex<u32>>,
    degree_a: usize,
    degree_b: usize,
) -> Option<(
    &'a Vec<NodeIndex<u32>>,
    &'a Vec<NodeIndex<u32>>,
    usize,
    usize,
)> {
    if all_nodes_with_degree(graph, node_indices_a, degree_a)
        && all_nodes_with_degree(graph, node_indices_b, degree_b)
    {
        Some((&node_indices_a, &node_indices_b, degree_a, degree_b))
    } else if all_nodes_with_degree(graph, node_indices_a, degree_b)
        && all_nodes_with_degree(graph, node_indices_b, degree_a)
    {
        Some((&node_indices_b, &node_indices_a, degree_a, degree_b))
    } else {
        None
    }
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
        generate_biregular_graphs(n, a, b).len()
    }

    #[test]
    fn test_biregular_graph_partitions_have_correct_degrees() {
        let graphs = generate_biregular_graphs(5, 3, 2);

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
}