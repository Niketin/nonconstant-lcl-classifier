use std::path::PathBuf;

use crate::GraphCache;

use super::get_partitions;
use super::{
    biregular_partition_sizes, extend_to_multigraphs,
    generate_bipartite_graphs_with_degree_bounds_graph8, generate_bipartite_multigraphs,
    multigraph_string_to_petgraph, partition_is_regular, UndirectedGraph,
};
//use crate::GraphCache;
use itertools::Itertools;
use log::debug;
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
    /// Generates nonisomorphic biregular graphs with parallel edges.
    ///
    /// Immediate results are cached into filesystem.
    /// The immediate results are the underlying bipartite graphs before extending them to multigraphs.
    ///
    /// Simple graphs are also included in this generators result.
    /// TODO rethink caching and which graph generator functions really need them.
    /// TODO Might bee wasteful to use it with "generate_bipartite_graphs_with_degree_bounds_graph8".
    pub fn generate_multigraphs(graph_size: usize, degree_a: usize, degree_b: usize) -> Vec<Self> {
        let max_degree = std::cmp::max(degree_a, degree_b);
        let max_edge_multiplicity = max_degree;

        let mut multigraphs = Vec::new();
        for (n1, n2) in biregular_partition_sizes(graph_size, degree_a, degree_b) {
            let path_to_graphs_in_cache =
                BiregularGraph::path_to_bipartite_graphs_cache(n1, n2, 1, 1, degree_a, degree_b);
            if !path_to_graphs_in_cache.exists() {
                BiregularGraph::generate_bipartite_graphs_to_cache(
                    &path_to_graphs_in_cache,
                    n1,
                    n2,
                    1,
                    1,
                    degree_a,
                    degree_b,
                    0,
                    0,
                )
                .expect("Generating graphs to cache failed.");
            } else {
                debug!(
                    "Found bipartite graph from cache! n1: {}, n2: {}, d1=[1...{}], d2=[1...{}]",
                    n1, n2, degree_a, degree_b
                )
            }

            let edges = n1 * degree_a;
            let mg = extend_to_multigraphs(
                &path_to_graphs_in_cache,
                max_edge_multiplicity,
                edges,
                max_degree,
            );
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

    /// Parallelly generates nonisomorphic biregular graphs with parallel edges.
    ///
    /// Graph generation is divided into multiple threads.
    /// After the threads are done, each subresult is combined into one collection of results.
    /// By default the function uses the amount of logical cores in the system.
    ///
    /// Multigraph results are cached using the `multigrap_cache`.
    /// Caching saves resources when multiple calls with the same class properties are given.
    pub fn get_or_generate_multigraphs_parallel<T: GraphCache>(
        graph_size: usize,
        degree_a: usize,
        degree_b: usize,
        multigraph_cache: Option<&mut T>,
        //simple_graph_cache: Option<impl GraphCache>,
    ) -> Vec<Self> {
        eprintln!("start get_or_generate_multigraphs_parallel");
        if let Some(cache) = &multigraph_cache {
            if let Ok(result) = cache.read_graphs(graph_size, degree_a, degree_b) {
                eprintln!("Found from cache!");
                return result;
            }
        }

        let multigraphs = Self::generate_multigraphs_parallel(graph_size, degree_a, degree_b);
        eprintln!("Generated multigraphs!");
        // Update cache
        if let Some(cache) = multigraph_cache {
            if let Ok(_) = cache.write_graphs(graph_size, degree_a, degree_b, &multigraphs) {
                eprintln!("Updated cache!");
            } else {
                eprintln!("Failed updating cache!");
            }
        }

        eprintln!("end get_or_generate_multigraphs_parallel");
        multigraphs
    }

    fn generate_multigraphs_parallel(
        graph_size: usize,
        degree_a: usize,
        degree_b: usize,
    ) -> Vec<Self> {
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

    fn path_to_bipartite_graphs_cache(
        n1: usize,
        n2: usize,
        d1_low: usize,
        d2_low: usize,
        d1_high: usize,
        d2_high: usize,
    ) -> PathBuf {
        let graph_cache_directory = "thesis_tool_graph_cache/multi";
        let mut path = dirs::home_dir().expect("User is expected to have a home directory.");

        path.push(graph_cache_directory);

        let graph_size_class = format!("{}_{}", n1, n2);
        path.push(graph_size_class);

        let file_name = format!("{}_{}_{}_{}", d1_low, d2_low, d1_high, d2_high);
        path.push(file_name);
        path.set_extension("dat");

        path
    }

    fn path_to_bipartite_graphs_tmp(
        n1: usize,
        n2: usize,
        d1_low: usize,
        d2_low: usize,
        d1_high: usize,
        d2_high: usize,
        result: usize,
        modulo: usize,
    ) -> PathBuf {
        assert!(result <= modulo);

        let mut path = PathBuf::new();
        path.push("/tmp");

        let graph_size_class = format!("{}_{}", n1, n2);
        path.push(graph_size_class);

        let file_name = format!(
            "{}_{}_{}_{}_{}_{}",
            d1_low, d2_low, d1_high, d2_high, result, modulo
        );
        path.push(file_name);
        path.set_extension("dat");

        path
    }

    fn generate_bipartite_graphs_to_cache(
        path: &PathBuf,
        n1: usize,
        n2: usize,
        d1_low: usize,
        d2_low: usize,
        d1_high: usize,
        d2_high: usize,
        result: usize,
        modulo: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let graphs = generate_bipartite_graphs_with_degree_bounds_graph8(
            n1, n2, d1_low, d2_low, d1_high, d2_high, result, modulo,
        );
        let mut dir = path.clone();
        dir.pop();
        std::fs::create_dir_all(dir)?;
        std::fs::write(path, &graphs)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generating_biregular_graphs_with_parallel_edges() {
        assert_eq!(BiregularGraph::generate_multigraphs(2, 2, 2).len(), 1);
        assert_eq!(BiregularGraph::generate_multigraphs(5, 2, 3).len(), 2);
        assert_eq!(BiregularGraph::generate_multigraphs(7, 3, 4).len(), 9);
        assert_eq!(BiregularGraph::generate_multigraphs(9, 8, 1).len(), 1);
    }

    #[test]
    fn test_biregular_graph_partitions_have_correct_degrees() {
        let graphs = BiregularGraph::generate_multigraphs(5, 3, 2);

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
    fn graph_eq<N, E, Ty, Ix>(
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
