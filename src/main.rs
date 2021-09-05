use graph6::string_to_adjacency_matrix;
use petgraph::{
    dot::{Config, Dot},
    graph::NodeIndex,
    visit::VisitMap,
    Graph, Undirected,
};
use std::{collections::HashSet, process::Command};

fn main() {
    let graphs = generate_biregular_graphs(9, 2, 3);

    graphs.into_iter().for_each(|x| {
        println!(
            "{}: {:?}, {}: {:?}",
            x.degree_a, x.partition_a, x.degree_b, x.partition_b
        );
        print_dot(&x.graph);
    })
}

struct BiregularGraph {
    graph: Graph<u32, (), Undirected, u32>,
    partition_a: Vec<NodeIndex>,
    partition_b: Vec<NodeIndex>,
    degree_a: usize,
    degree_b: usize,
}

fn generate_biregular_graphs(
    graph_size: usize,
    degree_a: usize,
    degree_b: usize,
) -> Vec<BiregularGraph> {
    let mut command = Command::new("geng");
    let graphs = command.arg("-bc").arg(graph_size.to_string());

    let out = graphs.output().expect("msg");
    let lines = String::from_utf8(out.stdout).expect("Not in utf8 format");

    let lines_it = lines.lines();

    let mut graphs_petgraph: Vec<BiregularGraph> = Vec::new();

    for line in lines_it {
        let adjacency_matrix = string_to_adjacency_matrix(line);
        let edges = adjacency_matrix_to_edge_list(adjacency_matrix);
        let graph: Graph<u32, (), Undirected, u32> = petgraph::graph::UnGraph::from_edges(&edges);
        let indices = &graph.node_indices().collect::<Vec<_>>();
        let res: Option<(Vec<NodeIndex>, Vec<NodeIndex>)> = is_bipartite(&graph, indices[0]);
        //println!("{:?}", res);
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
            graphs_petgraph.push(biregular_graph);
        }
    }

    return graphs_petgraph;
}

fn print_dot(graph: &Graph<u32, (), Undirected, u32>) {
    println!(
        "{:?}",
        Dot::with_config(&graph, &[Config::EdgeNoLabel, Config::NodeIndexLabel])
    );
}

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

pub fn is_bipartite(
    g: &Graph<u32, (), Undirected, u32>,
    start: NodeIndex<u32>,
) -> Option<(Vec<NodeIndex<u32>>, Vec<NodeIndex<u32>>)> {
    let mut red: HashSet<NodeIndex<u32>> = HashSet::with_capacity(g.node_count());
    red.visit(start);
    let mut blue: HashSet<NodeIndex<u32>> = HashSet::with_capacity(g.node_count());

    let mut stack = ::std::collections::VecDeque::new();
    stack.push_front(start);

    while let Some(node) = stack.pop_front() {
        let is_red = red.contains(&node);
        let is_blue = blue.contains(&node);

        assert!(is_red ^ is_blue);

        for neighbour in g.neighbors(node) {
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
                        panic!("Invariant doesn't hold");
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

/// Check if all nodes at node_indices have the specified degree or 1.
fn are_all_nodes_given_degree_or_1(
    graph: &Graph<u32, (), Undirected, u32>,
    node_indices: &Vec<NodeIndex<u32>>,
    degree: usize,
) -> bool {
    let allowed_degrees = [1, degree];
    return node_indices
        .into_iter()
        .all(|x| allowed_degrees.contains(&graph.neighbors(*x).count()));
}

/// Checks if the graph is biregular.
///
/// Graph is biregular if it is bipartite, and
/// nodes in set A have degree degree_a or 1
/// and
/// nodes in set B have degree degree_b or 1
///
/// Bipartity is assumed and not checked.
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
    if are_all_nodes_given_degree_or_1(graph, node_indices_a, degree_a)
        && are_all_nodes_given_degree_or_1(graph, node_indices_b, degree_b)
    {
        Some((&node_indices_a, &node_indices_b, degree_a, degree_b))
    } else if are_all_nodes_given_degree_or_1(graph, node_indices_a, degree_b)
        && are_all_nodes_given_degree_or_1(graph, node_indices_b, degree_a)
    {
        Some((&node_indices_a, &node_indices_b, degree_b, degree_a))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generating_biregular_graphs() {
        assert_eq!(count_biregular_graphs(9, 2, 3), 5);
        assert_eq!(count_biregular_graphs(9, 3, 2), 5);

        assert_eq!(count_biregular_graphs(7, 2, 3), 3);
        assert_eq!(count_biregular_graphs(7, 3, 2), 3);
    }

    fn count_biregular_graphs(n: usize, a: usize, b: usize) -> usize {
        let mut command = Command::new("geng");
        let graphs = command.arg("-bc").arg(n.to_string());

        let out = graphs.output().expect("msg");
        let lines = String::from_utf8(out.stdout).expect("Not in utf8 format");

        let lines_it = lines.lines();

        let mut count = 0;
        for line in lines_it {
            let adjacency_matrix = string_to_adjacency_matrix(line);
            let edges = adjacency_matrix_to_edge_list(adjacency_matrix);
            let graph: Graph<u32, (), Undirected, u32> =
                petgraph::graph::UnGraph::from_edges(&edges);
            let indices = &graph.node_indices().collect::<Vec<_>>();
            let res: Option<(Vec<NodeIndex>, Vec<NodeIndex>)> = is_bipartite(&graph, indices[0]);

            if let Some((node_indices_a, node_indices_b)) = res {
                match is_biregular(&graph, &node_indices_a, &node_indices_b, a, b) {
                    None => continue,
                    Some(_) => count += 1,
                };
            }
        }
        count
    }
}
