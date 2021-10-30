use petgraph::{
    dot::{Config, Dot},
    Graph, Undirected,
};
use std::fmt::Debug;

/// Trait for things that can have a representation in .dot format.
pub trait DotFormat {
    fn get_dot(&self) -> String;
}

/// Implement DotFormat for undirected graphs.
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
