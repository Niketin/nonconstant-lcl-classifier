use super::get_partitions;
use super::{
    biregular_partition_sizes, generate_bipartite_multigraphs, multigraph_string_to_petgraph,
    partition_is_regular, UndirectedGraph,
};
use crate::GraphCache;
use itertools::Itertools;
use log::{error, info};
use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};
use std::mem;
use std::sync::mpsc;
use std::thread;

/// Container for biregular graph.
///
/// Graph can contain parallel edges.
///
/// Has two partitions, `partition_a` and `partition_b`.
/// Nodes in `partition_a` have degree of `degree_a`.
/// Nodes in `partition_b` have degree of `degree_b`.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BiregularGraph {
    pub graph: UndirectedGraph,
    pub partition_a: Vec<NodeIndex>,
    pub partition_b: Vec<NodeIndex>,
    pub degree_a: usize,
    pub degree_b: usize,
}

impl BiregularGraph {
    /// Generates nonisomorphic biregular multigraphs in parallel and uses the provided cache.
    ///
    /// Uses `Self::generate` to generate the graphs.
    ///
    /// Multigraph results are cached using the `multigrap_cache`.
    /// Caching saves resources when multiple calls with the same class properties are given.
    pub fn get_or_generate<T: GraphCache>(
        graph_size: usize,
        degree_a: usize,
        degree_b: usize,
        multigraph_cache: Option<&mut T>,
        //simple_graph_cache: Option<impl GraphCache>,
    ) -> Vec<Self> {
        if let Some(cache) = &multigraph_cache {
            if let Ok(result) = cache.read_graphs(graph_size, degree_a, degree_b) {
                info!("Found the graphs from cache!");
                return result;
            }
        }

        let multigraphs = Self::generate(graph_size, degree_a, degree_b);
        // Update cache
        if let Some(cache) = multigraph_cache {
            if let Ok(_) = cache.write_graphs(graph_size, degree_a, degree_b, &multigraphs) {
                info!("Updated the cache!");
            } else {
                error!("Failed updating cache!");
            }
        }

        multigraphs
    }

    /// Generates nonisomorphic biregular multigraphs in parallel.
    ///
    /// Graph generation is divided into multiple threads.
    /// After the threads are done, each subresult is combined into one collection of results.
    /// By default the function uses the amount of logical cores in the system.
    pub fn generate(graph_size: usize, degree_a: usize, degree_b: usize) -> Vec<Self> {
        let max_degree = std::cmp::max(degree_a, degree_b);
        let max_edge_multiplicity = max_degree;
        let threads = num_cpus::get();

        let (sender, receiver) = mpsc::channel();
        for i in 0..threads {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut multigraphs: Vec<((usize, usize), String)> = Vec::new();

                for (n1, n2) in biregular_partition_sizes(graph_size, degree_a, degree_b) {
                    let edges = n1 * degree_a;
                    let mg = generate_bipartite_multigraphs(
                        n1,
                        n2,
                        1,
                        1,
                        degree_a,
                        degree_b,
                        i,
                        threads,
                        max_edge_multiplicity,
                        edges,
                        max_degree,
                    );
                    multigraphs.push(((n1, n2), mg));
                }

                // Multigraphs in petgraph format.
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
                    .filter(|(g, (p1, p2))| {
                        partition_is_regular(&g, &p1) && partition_is_regular(&g, &p2)
                    })
                    .map(|(graph, (partition_a, partition_b))| Self {
                        degree_a,
                        degree_b,
                        graph,
                        partition_a,
                        partition_b,
                    })
                    .collect_vec();

                sender.send(multigraphs_biregulargraph).unwrap();
            });
        }
        mem::drop(sender);

        receiver.into_iter().flatten().collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generating_biregular_graphs_with_parallel_edges() {
        assert_eq!(BiregularGraph::generate(2, 2, 2).len(), 1);
        assert_eq!(BiregularGraph::generate(5, 2, 3).len(), 2);
        assert_eq!(BiregularGraph::generate(7, 3, 4).len(), 9);
        assert_eq!(BiregularGraph::generate(9, 8, 1).len(), 1);
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

    /// The idea is from: https://github.com/petgraph/petgraph/issues/199#issuecomment-484077775
    fn _graph_eq<N, E, Ty, Ix>(
        a: &petgraph::Graph<N, E, Ty, Ix>,
        b: &petgraph::Graph<N, E, Ty, Ix>,
    ) -> bool
    where
        N: PartialEq,
        E: PartialEq,
        Ty: petgraph::EdgeType,
        Ix: petgraph::graph::IndexType + PartialEq,
    {
        let get_edges = |g: &petgraph::Graph<N, E, Ty, Ix>| {
            g.raw_edges()
                .iter()
                .map(|e| {
                    let mut v = [e.source(), e.target()];
                    v.sort();
                    let [v1, v2] = v;
                    (v1, v2)
                })
                .collect_vec()
        };
        get_edges(&a).eq(&get_edges(&b))
    }
}
