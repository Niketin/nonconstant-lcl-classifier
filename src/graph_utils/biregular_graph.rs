use std::time::Instant;

use itertools::Itertools;
use log::debug;
use petgraph::{graph::NodeIndex, Graph, Undirected};

use super::{generate_biregular_graphs_with_total_size_graph8, get_partitions, graph6_to_petgraph};

/// Container for biregular graph.
///
/// Has two partitions, `partition_a` and `partition_b`.
/// Nodes in `partition_a` have degree of `degree_a`.
/// Nodes in `partition_b` have degree of `degree_b`.
pub struct BiregularGraph {
    pub graph: Graph<u32, (), Undirected, u32>,
    pub partition_a: Vec<NodeIndex>,
    pub partition_b: Vec<NodeIndex>,
    pub degree_a: usize,
    pub degree_b: usize,
}

impl BiregularGraph {
    /// Generates simple nonisomorphic biregular graphs.
    pub fn generate(graph_size: usize, degree_a: usize, degree_b: usize) -> Vec<Self> {
        let now = Instant::now();
        let (partition_sizes, graphs_string): (Vec<(usize, usize)>, Vec<String>) =
            generate_biregular_graphs_with_total_size_graph8(graph_size, degree_a, degree_b)
                .iter()
                .cloned()
                .unzip();
        debug!(
            "Generated all bipartite graphs in {} s.",
            now.elapsed().as_secs_f32()
        );

        let now = Instant::now();
        let bipartite_graphs_grouped_with_partition_sizes = graphs_string
            .iter()
            .map(|x| x.lines().map(|line| graph6_to_petgraph(line)).collect_vec())
            .collect_vec();
        debug!(
            "Transformed {} graph6-formatted graphs to petgraphs in {} s.",
            bipartite_graphs_grouped_with_partition_sizes
                .iter()
                .map(|x| x.len())
                .sum::<usize>(),
            now.elapsed().as_secs_f32()
        );

        let now = Instant::now();
        let bipartite_graphs_with_partitions = bipartite_graphs_grouped_with_partition_sizes
            .into_iter()
            .enumerate()
            .map(|(i, graphs)| {
                graphs
                    .into_iter()
                    .map(|x| {
                        let (n1, n2) = partition_sizes[i];
                        let t = get_partitions(&x, n1, n2);
                        (x, t)
                    })
                    .collect_vec()
            })
            .flatten()
            .collect_vec();
        debug!(
            "Partitioned {} bipartite graphs in {} s.",
            bipartite_graphs_with_partitions.len(),
            now.elapsed().as_secs_f32()
        );

        bipartite_graphs_with_partitions
            .into_iter()
            .map(|(graph, (partition_a, partition_b))| BiregularGraph {
                degree_a,
                degree_b,
                graph,
                partition_a,
                partition_b,
            })
            .collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_graphs() {
        assert_eq!(BiregularGraph::generate(5, 2, 3).len(), 1);
    }
}
