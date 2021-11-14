pub mod configurations;

use configurations::Configurations;
use itertools::Itertools;
use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::File,
    hash::{Hash, Hasher},
    io::Write,
    path::PathBuf,
};

/// Locally Checkable Labeling problem for biregular graphs.
///
/// Contains configurations for active nodes and passive nodes.
/// Also contains a symbol map that is used in both configurations' initialization.
#[derive(Debug, Clone, Eq)]
pub struct LclProblem {
    pub active: Configurations,
    pub passive: Configurations,
    pub symbol_map: HashMap<String, u8>,
}

impl Hash for LclProblem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.active.hash(state);
        self.passive.hash(state);
    }
}

impl LclProblem {
    pub fn new(a: &str, p: &str) -> Result<LclProblem, Box<dyn std::error::Error>> {
        let mut symbol_map: HashMap<String, u8> = HashMap::new();
        Ok(LclProblem {
            active: Configurations::from_string(a, &mut symbol_map)?,
            passive: Configurations::from_string(p, &mut symbol_map)?,
            symbol_map,
        })
    }

    fn from_configurations(
        a: Vec<Vec<u8>>,
        p: Vec<Vec<u8>>,
    ) -> Result<LclProblem, Box<dyn std::error::Error>> {
        let symbol_map: HashMap<String, u8> = HashMap::new();
        Ok(LclProblem {
            active: Configurations::from_configuration_data(a)?,
            passive: Configurations::from_configuration_data(p)?,
            symbol_map,
        })
    }

    /// Checks if either active or passive partition is empty.
    fn contains_empty_partition(&self) -> bool {
        self.active.get_configurations().is_empty() || self.passive.get_configurations().is_empty()
    }

    /// Adapted from https://github.com/AleksTeresh/lcl-classifier/blob/be5d0196b02dad33ee19657af6b16457f59780e9/src/server/problem/problem.py#L378
    fn purge(&mut self) {
        let mut active_symbols = self.active.get_symbols_set();
        let mut passive_symbols = self.passive.get_symbols_set();

        while active_symbols
            .symmetric_difference(&passive_symbols)
            .next()
            .is_some()
        {
            let diff = active_symbols
                .difference(&passive_symbols)
                .copied()
                .collect_vec();
            if !diff.is_empty() {
                self.active.remove_configurations_containing_symbol(&diff);
            }

            let diff = passive_symbols
                .difference(&active_symbols)
                .copied()
                .collect_vec();
            if !diff.is_empty() {
                self.passive.remove_configurations_containing_symbol(&diff);
            }

            active_symbols = self.active.get_symbols_set();
            passive_symbols = self.passive.get_symbols_set();
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
    pub fn generate(active_degree: usize, passive_degree: usize, label_count: u8) -> Vec<Self> {
        let labels = (0..label_count).collect_vec();
        let generated_collections_of_active_configurations =
            Self::generate_all(active_degree, &labels);
        let generated_collections_of_passive_configurations =
            Self::generate_all(passive_degree, &labels);

        generated_collections_of_active_configurations
            .iter()
            .cartesian_product(generated_collections_of_passive_configurations.iter())
            .filter_map(|(active, passive)| {
                let mut problem =
                    LclProblem::from_configurations(active.clone(), passive.clone()).unwrap();
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
        let symbol_max = self
            .active
            .get_symbols()
            .into_iter()
            .chain(self.passive.get_symbols().into_iter())
            .max()
            .unwrap();
        let symbols = 0..=symbol_max;

        let permutations = symbols.permutations(symbol_max as usize + 1).collect_vec();

        let a = permutations.into_iter().map(|perm| {
            assert!(!perm.is_empty());
            let active = self.active.map_symbols(&perm);
            let passive = self.passive.map_symbols(&perm);
            return (active, passive);
        });

        return a.collect_vec();
    }

    fn generate_all(degree: usize, labels: &Vec<u8>) -> Vec<Vec<Vec<u8>>> {
        let configurations = Self::gen_configurations(degree, &labels);

        let iterator = (1..=configurations.len())
            .flat_map(|max_configurations| {
                configurations
                    .iter()
                    .cloned()
                    .combinations(max_configurations)
            })
            .collect_vec();

        return iterator;
    }

    fn gen_configurations(degree: usize, labels: &Vec<u8>) -> Vec<Vec<u8>> {
        labels
            .iter()
            .cloned()
            .combinations_with_replacement(degree)
            .collect_vec()
    }

    /// Writes problems to a file and removes old content.
    ///
    /// Creates the file if it does not exist in `path`.
    /// Problems are separeted with newline.
    /// Supports up to 7 different symbols.
    /// The symbols are the 7 first letters in the alphabet.
    ///
    /// An example of a problem:
    /// ```['AAB', 'AAC']; ['AB', 'AC] ```
    pub fn write_to_file(
        path: PathBuf,
        problems: &Vec<LclProblem>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(path)?;

        let symbols = "ABCDEFG";

        problems.iter().for_each(|ref problem| {
            let asd = [&problem.active, &problem.passive]
                .iter()
                .map(|problem_set| {
                    let mut conf =
                        problem_set
                            .get_configurations()
                            .into_iter()
                            .map(|configuration| {
                                let c = configuration
                                    .iter()
                                    .map(|&l| symbols.chars().nth(l as usize).unwrap())
                                    .collect_vec();
                                format!("\'{}\'", c.iter().join(""))
                            });
                    format!("[{}]", conf.join(", "))
                })
                .collect_vec();
            file.write(format!("{}\n", asd.join("; ")).as_bytes())
                .unwrap();
        });

        Ok(())
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

        assert_ne!(problem0.symbol_map, problem1.symbol_map);

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
