use petgraph::{graph::NodeIndex, Graph, Undirected};

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
