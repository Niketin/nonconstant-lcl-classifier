pub mod configurations;

use configurations::Configurations;
use itertools::Itertools;
use log::info;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::File,
    hash::{Hash, Hasher},
    io::Write,
    path::PathBuf,
};
use std::string::ToString;

use crate::caches::{lcl_problem::LclProblemCacheParams, Cache};

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
        let mut label_map: HashMap<char, u8> = HashMap::new();
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

    /// Removes redundant configurations.
    ///
    /// Configurations, that contain some label l such that l is not in any configuration
    /// on the other configuration set, are considered redundant.
    ///
    /// Adapted from <https://github.com/AleksTeresh/lcl-classifier/blob/be5d0196b02dad33ee19657af6b16457f59780e9/src/server/problem/problem.py#L378>
    pub fn purge(&mut self) {
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

    /// Generate all unique problems of a class (cached).
    ///
    /// TODO this is not really cached, and should be named accordingly.
    /// Uses `Self::purge` for each generated problem and
    /// removes problems with empty partition from the result.
    pub fn get_or_generate(
        active_degree: usize,
        passive_degree: usize,
        alphabet_length: u8,
    ) -> Vec<Self> {
        let active_configuration_powerset =
            Configurations::generate_powerset(active_degree, alphabet_length);

        let passive_configuration_powerset = if active_degree == passive_degree {
            None
        } else {
            Some(Configurations::generate_powerset(
                passive_degree,
                alphabet_length,
            ))
        };

        let cartesian_product = active_configuration_powerset.iter().cartesian_product(
            passive_configuration_powerset
                .as_ref()
                .unwrap_or(&active_configuration_powerset)
                .iter(),
        );

        cartesian_product
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
    /// Intermediate results (powersets) are cached using parameter `cache`.
    ///
    /// Generates problems with `Self::generate` and then normalizes them.
    /// Returns only unique problems.
    pub fn generate_normalized(
        active_degree: usize,
        passive_degree: usize,
        label_count: u8,
    ) -> Vec<Self> {
        let mut problems = Self::get_or_generate(active_degree, passive_degree, label_count);
        problems.iter_mut().for_each(|p| p.normalize());
        problems.into_iter().unique().collect_vec()
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
            (active, passive)
        });

        a.collect_vec()
    }

    /// Generate all unique normalized problems of a class (cached).
    ///
    /// Uses `Self::generate` to generate problems.
    pub fn get_or_generate_normalized<T: Cache<LclProblemCacheParams, LclProblem>>(
        active_degree: usize,
        passive_degree: usize,
        alphabet_length: u8,
        normalized_problem_cache: Option<&mut T>,
    ) -> Vec<Self> {
        let params = LclProblemCacheParams {
            degree_a: active_degree,
            degree_p: passive_degree,
            label_count: alphabet_length as usize,
        };
        if let Some(cache) = &normalized_problem_cache {
            if let Ok(result) = cache.read(params) {
                info!(
                    "Read the problems (deg_active={}, deg_passive={}, labels={}) from cache",
                    active_degree, passive_degree, alphabet_length
                );
                return result;
            }
        }

        let problems = Self::generate_normalized(active_degree, passive_degree, alphabet_length);
        // Update cache
        if let Some(cache) = normalized_problem_cache {
            cache
                .write(params,
                    &problems,
                ).unwrap_or_else(|_|
                panic!("Failed writing the problems (deg_active={}, deg_passive={}, labels={}) to cache",
                active_degree, passive_degree, alphabet_length));
            info!(
                "wrote the problems (deg_active={}, deg_passive={}, labels={}) to cache",
                active_degree, passive_degree, alphabet_length
            );
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
        problems: &[LclProblem],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(path)?;

        problems.iter().for_each(|problem| {
            let problem_string = problem.to_string();
            file.write_all(format!("{}\n", problem_string).as_bytes())
                .unwrap();
        });

        Ok(())
    }

}

impl ToString for LclProblem {
    /// Returns a string representation of the problem.
    ///
    /// Supports up to 26 different labels.
    /// The labels are the 26 letters in the English alphabet.
    ///
    /// An example of a problem:
    /// ```AAB AAC; AB AC```
    fn to_string(&self) -> String {
        let labels = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let configurations = [&self.active, &self.passive];
        let configurations_string = configurations
            .iter()
            .map(|problem_set| {
                let mut conf = problem_set
                    .get_configurations()
                    .iter()
                    .map(|configuration| {
                        configuration
                            .iter()
                            .map(|&l| labels.chars().nth(l as usize).unwrap())
                            .join("")
                    });
                conf.join(" ")
            })
            .collect_vec();
        configurations_string.join("; ")
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

pub trait Purgeable<T> {
    fn purge(self) -> Vec<T>;
}

pub trait Normalizable<T> {
    fn normalize(self) -> Vec<T>;
}

impl Purgeable<LclProblem> for Vec<LclProblem> {
    fn purge(self) -> Vec<LclProblem> {
        self.into_iter()
            .filter_map(|mut problem| {
                problem.purge();
                if !problem.contains_empty_partition() {
                    return Some(problem);
                }
                None
            })
            .unique()
            .collect_vec()
    }
}

impl Normalizable<LclProblem> for Vec<LclProblem> {
    fn normalize(self) -> Vec<LclProblem> {
        self.into_iter()
            .update(|p| p.normalize())
            .unique()
            .collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use crate::caches::LclProblemSqliteCache;

    use super::*;

    #[test]
    fn test_new_lcl_problem() {
        const A: &'static str = "MUU PPP";
        const P: &'static str = "MM PU UU";

        let problem = LclProblem::new(A, P);

        assert_eq!(problem.is_ok(), true);
    }

    #[test]
    fn test_normalize() {
        const A0: &'static str = "MUU PPP";
        const P0: &'static str = "MM PU UU";
        let mut problem0 = LclProblem::new(A0, P0).unwrap();

        const A1: &'static str = "XXX UUM";
        const P1: &'static str = "MM XU UU";
        let mut problem1 = LclProblem::new(A1, P1).unwrap();

        const A2: &'static str = "PPP UUU";
        const P2: &'static str = "MM PU UU";
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
        let problems = LclProblem::get_or_generate(3, 2, 3);
        assert_eq!(problems.len(), 44343)
    }

    #[test]
    fn test_normalized_problems_count_0() {
        let problems =
            LclProblem::get_or_generate_normalized::<LclProblemSqliteCache>(2, 1, 2, None);
        assert_eq!(problems.len(), 5);
    }

    #[test]
    fn test_normalized_problems_count_1() {
        let problems =
            LclProblem::get_or_generate_normalized::<LclProblemSqliteCache>(3, 2, 3, None);
        assert_eq!(problems.len(), 7735);
    }

    pub fn generate(
        active_degree: usize,
        passive_degree: usize,
        alphabet_length: u8,
    ) -> Vec<LclProblem> {
        let active_configuration_powerset =
            Configurations::generate_powerset(active_degree, alphabet_length);

        let passive_configuration_powerset = Configurations::generate_powerset(
            passive_degree,
            alphabet_length,
        );

        let cartesian_product = active_configuration_powerset.iter().cartesian_product(
            passive_configuration_powerset
                .iter(),
        );

        let purged = cartesian_product
            .map(|(active, passive)| {
                LclProblem::from_configurations(active.clone(), passive.clone())
            })
            .collect_vec();

        purged
    }

    #[test]
    fn test_generate() {
        assert_eq!(generate(1,1,1).len(), 1);
        assert_eq!(generate(1,1,2).len(), 9);
        assert_eq!(generate(2,1,2).len(), 21);
        assert_eq!(generate(2,2,2).len(), 49);
        assert_eq!(generate(3,2,2).len(), 105);
        assert_eq!(generate(4,2,2).len(), 217);
        assert_eq!(generate(4,3,2).len(), 465);
        assert_eq!(generate(4,4,2).len(), 961);
        assert_eq!(generate(6,6,2).len(), 16129);
        assert_eq!(generate(2,2,3).len(), 3969);
        assert_eq!(generate(3,2,3).len(), 64449);
        //assert_eq!(generate(3,3,3).len(), 1046529);
    }

    #[test]
    fn test_gen_purged() {
        assert_eq!(LclProblem::get_or_generate(1,1,1).len(), 1);
        assert_eq!(LclProblem::get_or_generate(1,1,2).len(), 3);
        assert_eq!(LclProblem::get_or_generate(2,1,2).len(), 7);
        assert_eq!(LclProblem::get_or_generate(2,2,2).len(), 27);
        assert_eq!(LclProblem::get_or_generate(3,2,2).len(), 67);
        assert_eq!(LclProblem::get_or_generate(4,2,2).len(), 147);
        assert_eq!(LclProblem::get_or_generate(4,3,2).len(), 379);
        assert_eq!(LclProblem::get_or_generate(4,4,2).len(), 843);
        assert_eq!(LclProblem::get_or_generate(6,6,2).len(), 15627);
        assert_eq!(LclProblem::get_or_generate(2,2,3).len(), 2103);
        assert_eq!(LclProblem::get_or_generate(3,2,3).len(), 44343);
        //assert_eq!(LclProblem::get_or_generate(3,3,3).len(), 962871);
    }

    #[test]
    fn test_gen_normalized() {
        assert_eq!(LclProblem::generate_normalized(1,1,1).len(), 1);
        assert_eq!(LclProblem::generate_normalized(1,1,2).len(), 2);
        assert_eq!(LclProblem::generate_normalized(2,1,2).len(), 5);
        assert_eq!(LclProblem::generate_normalized(2,2,2).len(), 18);
        assert_eq!(LclProblem::generate_normalized(3,2,2).len(), 38);
        assert_eq!(LclProblem::generate_normalized(4,2,2).len(), 84);
        assert_eq!(LclProblem::generate_normalized(4,3,2).len(), 200);
        assert_eq!(LclProblem::generate_normalized(4,4,2).len(), 446);
        assert_eq!(LclProblem::generate_normalized(6,6,2).len(), 7926);
        assert_eq!(LclProblem::generate_normalized(2,2,3).len(), 419);
        assert_eq!(LclProblem::generate_normalized(3,2,3).len(), 7735);
        //assert_eq!(LclProblem::generate_normalized(3,3,3).len(), 162299);
    }
}
