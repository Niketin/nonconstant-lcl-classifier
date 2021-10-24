use std::time::Instant;

use super::{
    biregular_partition_sizes, extend_to_multigraphs,
    generate_bipartite_graphs_with_degree_bounds_graph8, multigraph_string_to_petgraph,
    partition_is_regular, UndirectedGraph,
};
use super::{generate_biregular_graphs_unzipped_graph8, get_partitions, graph6_to_petgraph};
use itertools::Itertools;
use log::debug;
use petgraph::graph::NodeIndex;

/// Container for biregular graph.
///
/// Graph can contain parallel edges.
///
/// Has two partitions, `partition_a` and `partition_b`.
/// Nodes in `partition_a` have degree of `degree_a`.
/// Nodes in `partition_b` have degree of `degree_b`.
pub struct BiregularGraph {
    pub graph: UndirectedGraph,
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
            generate_biregular_graphs_unzipped_graph8(graph_size, degree_a, degree_b);

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
            .map(|(graph, (partition_a, partition_b))| Self {
                degree_a,
                degree_b,
                graph,
                partition_a,
                partition_b,
            })
            .collect_vec()
    }

    /// Generates nonisomorphic biregular graphs with parallel edges.
    ///
    /// Graphs with no parallel edges are not included. To generate them, use `generate`.
    pub fn generate_multigraph(graph_size: usize, degree_a: usize, degree_b: usize) -> Vec<Self> {
        let max_degree = std::cmp::max(degree_a, degree_b);
        let max_edge_multiplicity = max_degree;

        let mut multigraphs = Vec::new();
        for (n1, n2) in biregular_partition_sizes(graph_size, degree_a, degree_b) {
            let underlying_simple_bipartite_graphs =
                generate_bipartite_graphs_with_degree_bounds_graph8(
                    n1, n2, 1, 1, degree_a, degree_b,
                );
            let path = "/tmp/thesis-tool-graphs"; // TODO change to use some other method than save graphs in a file.
            std::fs::write(path, underlying_simple_bipartite_graphs).unwrap();
            let edges = n1 * degree_a;
            let mg = extend_to_multigraphs(path, max_edge_multiplicity, edges, max_degree);
            multigraphs.push(((n1, n2), mg));
        }

        let multigraphs_petgraph = multigraphs
            .into_iter()
            .filter_map(|((n1, n2), graphs)| {
                if let Ok(gs) = multigraph_string_to_petgraph(graphs) {
                    return Some(((n1, n2), gs));
                }
                None
            })
            .collect_vec();

        let multigraphs_biregulargraph = multigraphs_petgraph
            .into_iter()
            .map(|((n1, n2), graphs)| {
                graphs
                    .into_iter()
                    .map(|graph| {
                        let partitions = get_partitions(&graph, n1, n2);
                        (graph, partitions)
                    })
                    .collect_vec()
            })
            .flatten()
            .filter(|(g, (p1, p2))| partition_is_regular(&g, &p1) && partition_is_regular(&g, &p2))
            .map(|(graph, (partition_a, partition_b))| Self {
                degree_a,
                degree_b,
                graph,
                partition_a,
                partition_b,
            })
            .collect_vec();
        multigraphs_biregulargraph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generating_biregular_graphs() {
        assert_eq!(BiregularGraph::generate(5, 3, 2).len(), 1);
        assert_eq!(BiregularGraph::generate(5, 2, 3).len(), 1);
        assert_eq!(BiregularGraph::generate(7, 2, 3).len(), 0);
        assert_eq!(BiregularGraph::generate(7, 3, 2).len(), 0);
        assert_eq!(BiregularGraph::generate(8, 5, 3).len(), 1);
        assert_eq!(BiregularGraph::generate(8, 3, 5).len(), 1);
        assert_eq!(BiregularGraph::generate(8, 3, 3).len(), 1);
    }

    #[test]
    fn test_generating_biregular_graphs_with_parallel_edges() {
        assert_eq!(BiregularGraph::generate_multigraph(2, 2, 2).len(), 1);
        assert_eq!(BiregularGraph::generate_multigraph(5, 2, 3).len(), 2);
        assert_eq!(BiregularGraph::generate_multigraph(7, 3, 4).len(), 9);
        assert_eq!(BiregularGraph::generate_multigraph(9, 8, 1).len(), 1);
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
}
