pub mod lcl_problem_cache;
pub mod powerset_cache;

use crate::{Configurations, LclProblem};
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

pub trait PowersetCache {
    fn read_powerset(
        &self,
        degree: usize,
        label_count: usize,
    ) -> Result<Vec<Configurations>, Box<dyn std::error::Error>>;
    fn has_path(&self) -> bool;
    fn get_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>>;
    fn write_powerset(
        &mut self,
        degree: usize,
        label_count: usize,
        powerset: &Vec<Configurations>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
