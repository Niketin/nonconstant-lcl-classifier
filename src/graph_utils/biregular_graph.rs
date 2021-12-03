use std::path::PathBuf;
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
#[derive(Clone)]
pub struct BiregularGraph {
    pub graph: UndirectedGraph,
    pub partition_a: Vec<NodeIndex>,
    pub partition_b: Vec<NodeIndex>,
    pub degree_a: usize,
    pub degree_b: usize,
}

impl BiregularGraph {
    /// Generates simple nonisomorphic biregular graphs.
    pub fn generate_simple(graph_size: usize, degree_a: usize, degree_b: usize) -> Vec<Self> {
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
                BiregularGraph::set_bipartite_graphs_to_cache(
                    &path_to_graphs_in_cache,
                    n1,
                    n2,
                    1,
                    1,
                    degree_a,
                    degree_b,
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

    fn path_to_bipartite_graphs_cache(
        n1: usize,
        n2: usize,
        d1_low: usize,
        d2_low: usize,
        d1_high: usize,
        d2_high: usize,
    ) -> PathBuf {
        let graph_cache_directory = "thesis_tool_graph_cache";
        let mut path = dirs::home_dir().expect("User is expected to have a home directory.");

        path.push(graph_cache_directory);

        let graph_size_class = format!("{}_{}", n1, n2);
        path.push(graph_size_class);

        let file_name = format!("{}_{}_{}_{}", d1_low, d2_low, d1_high, d2_high);
        path.push(file_name);
        path.set_extension("dat");

        path
    }

    fn set_bipartite_graphs_to_cache(
        path: &PathBuf,
        n1: usize,
        n2: usize,
        d1_low: usize,
        d2_low: usize,
        d1_high: usize,
        d2_high: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let graphs = generate_bipartite_graphs_with_degree_bounds_graph8(
            n1, n2, d1_low, d2_low, d1_high, d2_high,
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
    fn test_generating_biregular_graphs() {
        assert_eq!(BiregularGraph::generate_simple(5, 3, 2).len(), 1);
        assert_eq!(BiregularGraph::generate_simple(5, 2, 3).len(), 1);
        assert_eq!(BiregularGraph::generate_simple(7, 2, 3).len(), 0);
        assert_eq!(BiregularGraph::generate_simple(7, 3, 2).len(), 0);
        assert_eq!(BiregularGraph::generate_simple(8, 5, 3).len(), 1);
        assert_eq!(BiregularGraph::generate_simple(8, 3, 5).len(), 1);
        assert_eq!(BiregularGraph::generate_simple(8, 3, 3).len(), 1);
    }

    #[test]
    fn test_generating_biregular_graphs_with_parallel_edges() {
        assert_eq!(BiregularGraph::generate_multigraphs(2, 2, 2).len(), 1);
        assert_eq!(BiregularGraph::generate_multigraphs(5, 2, 3).len(), 2);
        assert_eq!(BiregularGraph::generate_multigraphs(7, 3, 4).len(), 9);
        assert_eq!(BiregularGraph::generate_multigraphs(9, 8, 1).len(), 1);
    }

    #[test]
    fn test_biregular_graph_partitions_have_correct_degrees() {
        let graphs = BiregularGraph::generate_simple(5, 3, 2);

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

    #[test]
    fn test_multigraph_gen_includes_all_simple_graphs() {
        let graph_size = 5;
        let degree_a = 2;
        let degree_b = 3;

        let simple_graphs = BiregularGraph::generate_simple(graph_size, degree_a, degree_b);
        let multigraphs = BiregularGraph::generate_multigraphs(graph_size, degree_a, degree_b);

        for sg in simple_graphs.iter() {
            assert!(multigraphs.iter().any(|mg| graph_eq(&sg.graph, &mg.graph)))
        }
    }
}
