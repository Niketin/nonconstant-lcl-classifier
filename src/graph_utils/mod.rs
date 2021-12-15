mod biregular_graph;
mod dot_format;

pub use biregular_graph::BiregularGraph;
pub use dot_format::DotFormat;
use itertools::Itertools;
use petgraph::{graph::NodeIndex, Graph, Undirected};
use std::io::prelude::*;
use std::{fs::File, process::Command, process::Stdio};

pub type UndirectedGraph = Graph<u32, (), Undirected>;

/// Writes dot formatted graph into svg file.
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

/// Returns all positive integer pairs that sum up to `sum`.
///
/// First integer is always smaller or equal with the second.
fn pairs_with_sum(sum: usize) -> Vec<(usize, usize)> {
    (1..=(sum / 2)).map(|i| (i, sum - i)).collect_vec()
}

/// Returns all possible partition sizes of a biregular graph.
///
/// To be more exact, the graph is (`d1`, `d2`)-biregular graph of size `n`.
fn biregular_partition_sizes(n: usize, d1: usize, d2: usize) -> Vec<(usize, usize)> {
    pairs_with_sum(n)
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

/// Returns the partitions of a bipartite graph.
///
/// Assumes that the graph is bipartite and nodes are ordered by the partition
/// i.e. partition_A contains nodes 0..n1-1 (inclusive)
/// and partition_B contains nodes n1..n1+n2-1 (inclusive).
fn get_partitions(
    graph: &UndirectedGraph,
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

fn generate_bipartite_multigraphs(
    n1: usize,
    n2: usize,
    d1_low: usize,
    d2_low: usize,
    d1_high: usize,
    d2_high: usize,
    result: usize,
    modulo: usize,
    max_edge_multiplicity: usize,
    edges: usize,
    max_degree: usize,
) -> String {
    assert!(result <= modulo);

    let parameter_degree_lower_bound = format!("-d{}:{}", d1_low, d2_low);
    let parameter_degree_upper_bound = format!("-D{}:{}", d1_high, d2_high);

    // Use gengbg and assume it exists in the system.
    // Flag -c limits the output to connected graphs.
    let genbg_child = Command::new("genbg")
        .arg("-c")
        .arg(parameter_degree_lower_bound)
        .arg(parameter_degree_upper_bound)
        .arg(n1.to_string())
        .arg(n2.to_string())
        .arg(format!("{}/{}", result, modulo))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start genbg process");
    let genbg_out = genbg_child.stdout.expect("Failed to open genbg stdout");

    // Use multig and assume it exists in the system.
    let mut cmd_multig = Command::new("multig");

    let multig_child = cmd_multig
        .arg(format!("-e{}", edges))
        .arg(format!("-D{}", max_degree))
        .arg(format!("-m{}", max_edge_multiplicity))
        .arg("-T")
        .stdin(genbg_out);

    //.expect("Failed to start multig process");

    let out = multig_child.output().expect("Failed to open multig stdout");
    String::from_utf8(out.stdout).expect("Not in utf8 format")
}

fn multigraph_string_to_petgraph(
    multigraph_string: String,
) -> Result<Vec<UndirectedGraph>, Box<dyn std::error::Error>> {
    let mut graphs: Vec<UndirectedGraph> = vec![];

    for line in multigraph_string.lines() {
        let words = line.split_ascii_whitespace();

        let mut values = words.map(|word| word.parse::<u32>());

        let _number_of_vertices = values.next().ok_or("Invalid format of multigraph.")??;
        let number_of_edges = values.next().ok_or("Invalid format of multigraph.")??;

        if number_of_edges == 0 {
            continue;
        }

        let mut edges = vec![];

        for (v1, v2, mul) in values.tuples() {
            let v1 = v1?;
            let v2 = v2?;
            for _ in 0..mul? {
                edges.push((v1, v2));
            }
        }

        let graph: UndirectedGraph = petgraph::graph::UnGraph::from_edges(&edges);
        graphs.push(graph);
    }

    Ok(graphs)
}

fn partition_is_regular(graph: &UndirectedGraph, partition: &Vec<NodeIndex>) -> bool {
    let degrees = partition
        .iter()
        .map(|node| graph.neighbors(*node).count())
        .collect_vec();
    degrees.windows(2).all(|window| window[0] == window[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_indices(x: &[usize], g: &UndirectedGraph) -> Vec<NodeIndex> {
        x.iter()
            .map(|i| g.node_indices().find(|x| x.index() == *i).unwrap())
            .collect_vec()
    }

    #[test]
    fn test_b_sums() {
        assert_eq!(pairs_with_sum(3), vec![(1, 2)]);
        assert_eq!(pairs_with_sum(4), vec![(1, 3), (2, 2)]);
        assert_eq!(pairs_with_sum(5), vec![(1, 4), (2, 3)]);
    }

    #[test]
    fn test_biregular_partition_sizes() {
        assert_eq!(biregular_partition_sizes(5, 2, 3).len(), 1);
        assert_eq!(biregular_partition_sizes(5, 3, 2).len(), 1);
    }

    #[test]
    fn test_partition_is_regular() {
        let edges = vec![(0, 1), (0, 1), (1, 2), (1, 2)];

        let graph: UndirectedGraph = petgraph::graph::UnGraph::from_edges(edges);

        let p1 = get_indices(&[0, 2], &graph);
        let p2 = get_indices(&[1], &graph);

        for partition in [p1, p2] {
            assert!(partition_is_regular(&graph, &partition))
        }

        let p3 = [0, 1]
            .iter()
            .map(|i| graph.node_indices().find(|x| x.index() == *i).unwrap())
            .collect_vec();

        assert!(!partition_is_regular(&graph, &p3));
    }

    #[test]
    fn test_partition_is_regular2() {
        let edges = vec![(0, 2), (0, 3), (0, 4), (1, 2), (1, 3), (1, 4)];

        let graph: UndirectedGraph = petgraph::graph::UnGraph::from_edges(edges);

        let p1 = get_indices(&[0, 1], &graph);
        let p2 = get_indices(&[2, 3, 4], &graph);

        for partition in [p1, p2] {
            assert!(partition_is_regular(&graph, &partition))
        }
    }
}
