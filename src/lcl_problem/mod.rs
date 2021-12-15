pub mod configurations;

use configurations::Configurations;
use itertools::Itertools;
use log::{info, error};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::File,
    hash::{Hash, Hasher},
    io::Write,
    path::PathBuf,
};

use crate::caches::lcl_problem::LclProblemCache;

/// Locally Checkable Labeling problem for biregular graphs.
///
/// Contains configurations for active nodes and passive nodes.
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct LclProblem {
    pub active: Configurations,
    pub passive: Configurations,
}

impl Hash for LclProblem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.active.hash(state);
        self.passive.hash(state);
    }
}

impl LclProblem {
    pub fn new(a: &str, p: &str) -> Result<LclProblem, Box<dyn std::error::Error>> {
        let mut label_map: HashMap<String, u8> = HashMap::new();
        Ok(LclProblem {
            active: Configurations::from_string(a, &mut label_map)?,
            passive: Configurations::from_string(p, &mut label_map)?,
        })
    }

    fn from_configurations(active: Configurations, passive: Configurations) -> Self {
        Self { active, passive }
    }

    /// Checks if either active or passive partition is empty.
    fn contains_empty_partition(&self) -> bool {
        self.active.get_configurations().is_empty() || self.passive.get_configurations().is_empty()
    }

    /// Adapted from https://github.com/AleksTeresh/lcl-classifier/blob/be5d0196b02dad33ee19657af6b16457f59780e9/src/server/problem/problem.py#L378
    fn purge(&mut self) {
        let mut active_labels = self.active.get_labels_set();
        let mut passive_labels = self.passive.get_labels_set();

        while active_labels
            .symmetric_difference(&passive_labels)
            .next()
            .is_some()
        {
            let diff = active_labels
                .difference(&passive_labels)
                .copied()
                .collect_vec();
            if !diff.is_empty() {
                self.active.remove_configurations_containing_label(&diff);
            }

            let diff = passive_labels
                .difference(&active_labels)
                .copied()
                .collect_vec();
            if !diff.is_empty() {
                self.passive.remove_configurations_containing_label(&diff);
            }

            active_labels = self.active.get_labels_set();
            passive_labels = self.passive.get_labels_set();
        }
    }

    pub fn normalize(&mut self) {
        let mut problems = self.get_all_permutations();

        problems.iter_mut().for_each(|(a, p)| {
            a.sort();
            p.sort();
        });

        // Pick the lexicographically first problem.
        let mut first = problems
            .into_iter()
            .min_by(|(a0, p0), (a1, p1)| a0.cmp(a1).then_with(|| p0.cmp(p1)))
            .unwrap();

        // Swap configurations with the first problem.
        std::mem::swap(&mut first.0, &mut self.active);
        std::mem::swap(&mut first.1, &mut self.passive);
    }

    /// Generate all unique problems of a class.
    ///
    /// Uses `Self::purge` for each generated problem and
    /// removes problems with empty partition from the result.
    pub fn generate(active_degree: usize, passive_degree: usize, alphabet_length: u8) -> Vec<Self> {
        // TODO generate these in parallel and make it cached. These are the power sets.
        let generated_collections_of_active_configurations =
        Configurations::generate_all(active_degree, alphabet_length);

        // TODO generate these in parallel and make it cached. These are the power sets.
        let generated_collections_of_passive_configurations =
            Configurations::generate_all(passive_degree, alphabet_length);

        // TODO generate these in parallel and make it cached.
        generated_collections_of_active_configurations
            .iter()
            .cartesian_product(generated_collections_of_passive_configurations.iter())
            .filter_map(|(active, passive)| {
                let mut problem = LclProblem::from_configurations(active.clone(), passive.clone());
                problem.purge();
                if !problem.contains_empty_partition() {
                    return Some(problem);
                }
                None
            })
            .unique()
            .collect_vec()
    }

    /// Generates all unique normalized problems of a class.
    ///
    /// Generates problems with `Self::generate` and then normalizes them.
    /// Returns only unique problems.
    pub fn generate_normalized(
        active_degree: usize,
        passive_degree: usize,
        label_count: u8,
    ) -> Vec<Self> {
        let mut problems = Self::generate(active_degree, passive_degree, label_count);
        problems.iter_mut().for_each(|p| p.normalize());
        return problems.into_iter().unique().collect_vec();
    }

    fn get_all_permutations(&self) -> Vec<(Configurations, Configurations)> {
        let label_max = self
            .active
            .get_labels()
            .into_iter()
            .chain(self.passive.get_labels().into_iter())
            .max()
            .unwrap();
        let labels = 0..=label_max;

        let permutations = labels.permutations(label_max as usize + 1).collect_vec();

        let a = permutations.into_iter().map(|perm| {
            assert!(!perm.is_empty());
            let active = self.active.map_labels(&perm);
            let passive = self.passive.map_labels(&perm);
            return (active, passive);
        });

        return a.collect_vec();
    }

    /// Generate all unique normalized problems of a class (cached).
    ///
    /// Uses `Self::generate` to generate problems.
    pub fn get_or_generate_normalized<T: LclProblemCache>(active_degree: usize, passive_degree: usize, alphabet_length: u8, cache: Option<&mut T>) -> Vec<Self> {
        if let Some(cache) = &cache {
            if let Ok(result) = cache.read_problems(active_degree, passive_degree, alphabet_length as usize) {
                info!("Found the problems from the cache!");
                return result;
            }
        }

        let problems = Self::generate_normalized(active_degree, passive_degree, alphabet_length);
        // Update cache
        if let Some(cache) = cache {
            if let Ok(_) = cache.write_problems(active_degree, passive_degree, alphabet_length as usize, &problems) {
                info!("Wrote new problems to the cache!");
            } else {
                error!("Failed writing problems to the cache!");
            }
        }

        problems
    }

    /// Writes problems to a file and removes old content.
    ///
    /// Creates the file if it does not exist in `path`.
    /// Problems are converted to strings using `LclProblem::to_string`
    /// and are separeted with newline.
    pub fn write_to_file(
        path: PathBuf,
        problems: &Vec<LclProblem>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(path)?;

        problems.iter().for_each(|ref problem| {
            let problem_string = problem.to_string();
            file.write(format!("{}\n", problem_string).as_bytes())
                .unwrap();
        });

        Ok(())
    }

    /// Returns a string representation of the problem.
    ///
    /// Supports up to 7 different labels.
    /// The labels are the 7 first letters in the alphabet.
    ///
    /// An example of a problem:
    /// ```['AAB', 'AAC']; ['AB', 'AC] ```
    pub fn to_string(&self) -> String {
        let labels = "ABCDEFG";
        let configurations = [&self.active, &self.passive];
        let configurations_string = configurations
            .iter()
            .map(|problem_set| {
                let mut conf = problem_set
                    .get_configurations()
                    .into_iter()
                    .map(|configuration| {
                        let c = configuration
                            .iter()
                            .map(|&l| labels.chars().nth(l as usize).unwrap())
                            .collect_vec();
                        format!("\'{}\'", c.iter().join(""))
                    });
                format!("[{}]", conf.join(", "))
            })
            .collect_vec();
        format!("{}", configurations_string.join("; "))
    }
}

impl PartialEq for LclProblem {
    fn eq(&self, other: &Self) -> bool {
        self.active == other.active && self.passive == other.passive
    }
}

impl PartialOrd for LclProblem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.active
                .cmp(&other.active)
                .then_with(|| self.passive.cmp(&other.passive)),
        )
    }
}

impl Ord for LclProblem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.active
            .cmp(&other.active)
            .then_with(|| self.passive.cmp(&other.passive))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lcl_problem() {
        const A: &'static str = "M U U\nP P P";
        const P: &'static str = "M M\nP U\nU U";

        let problem = LclProblem::new(A, P);

        assert_eq!(problem.is_ok(), true);
    }

    #[test]
    fn test_normalize() {
        const A0: &'static str = "M U U\nP P P";
        const P0: &'static str = "M M\nP U\nU U";
        let mut problem0 = LclProblem::new(A0, P0).unwrap();

        const A1: &'static str = "X X X\n U U M";
        const P1: &'static str = "M M\nX U\nU U";
        let mut problem1 = LclProblem::new(A1, P1).unwrap();

        const A2: &'static str = "P P P\n U U U";
        const P2: &'static str = "M M\nP U\nU U";
        let mut problem2 = LclProblem::new(A2, P2).unwrap();

        assert_ne!(problem0, problem1);
        assert_ne!(problem0, problem2);
        assert_ne!(problem1, problem2);

        problem0.normalize();
        problem1.normalize();
        problem2.normalize();

        assert_eq!(problem0, problem1);

        assert_ne!(problem0, problem2);
        assert_ne!(problem1, problem2);
    }

    #[test]
    fn test_problems_count() {
        let problems = LclProblem::generate(3, 2, 3);
        assert_eq!(problems.len(), 44343)
    }

    #[test]
    fn test_normalized_problems_count_0() {
        let problems = LclProblem::generate_normalized(2, 1, 2);
        assert_eq!(problems.len(), 5);
    }

    #[test]
    fn test_normalized_problems_count_1() {
        let problems = LclProblem::generate_normalized(3, 2, 3);
        assert_eq!(problems.len(), 7735);
    }
}
