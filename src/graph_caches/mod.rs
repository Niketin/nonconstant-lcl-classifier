use std::path::PathBuf;

use crate::BiregularGraph;

pub mod multigraph_cache;
pub mod simple_graph_cache;

pub trait GraphCache {
    fn read_graphs(
        &self,
        n: usize,
        degree_a: usize,
        degree_p: usize,
    ) -> Result<Vec<BiregularGraph>, Box<dyn std::error::Error>>;
    fn has_path(&self) -> bool;
    fn get_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>>;
    fn write_graphs(
        &mut self,
        nodes: usize,
        degree_a: usize,
        degree_p: usize,
        graphs: &Vec<BiregularGraph>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
