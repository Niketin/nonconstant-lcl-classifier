pub mod lcl_problem_cache;

use crate::LclProblem;
use std::path::PathBuf;

pub trait LclProblemCache {
    fn read_problems(
        &self,
        degree_a: usize,
        degree_p: usize,
        label_count: usize,
    ) -> Result<Vec<LclProblem>, Box<dyn std::error::Error>>;
    fn has_path(&self) -> bool;
    fn get_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>>;
    fn write_problems(
        &mut self,
        degree_a: usize,
        degree_p: usize,
        label_count: usize,
        problems: &Vec<LclProblem>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
