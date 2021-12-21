use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    error::Error,
    iter::FromIterator,
};

/// A container for set of configurations that are used to define an LCL problem.
///
/// A configuration is a multiset of labels.
/// A new Configuration can be created by using method [`Configurations::new`].
///
/// Contained configurations can be accessed with different methods.
/// It is also possible to access all unique permutations of each configuration with [`Configurations::get_permutations`].
#[derive(Debug, Clone, Eq, Ord, Hash, Serialize, Deserialize)]
pub struct Configurations {
    data: Vec<Vec<u8>>,
}

impl Configurations {
    /// Creates Configuration instance from given `encoding` using given `label_map`.
    ///
    /// Encoding is formated as a multirow string where each configuration is separated with linebreak and each label is separated with space.
    /// Each configuration has to be equally long.
    ///
    /// Internally each label is mapped to unsigned integers and then saved in vector as `u8`.
    /// By default, labels increase starting from 0.
    /// A label_map is supposed to be given if it is desired to have multiple [`Configurations`] instances using same mapping of labels.
    ///
    ///
    /// # Example
    /// ```
    /// use std::collections::HashMap;
    /// # use thesis_tool_lib::Configurations;
    /// let mut label_map = HashMap::<String, u8>::new();
    /// let configurations = Configurations::from_string("A B C\nA A B\nC C C", &mut label_map).unwrap();
    /// ```
    pub fn from_string(
        encoding: &str,
        label_map: &mut HashMap<String, u8>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut lines = encoding.lines();
        let first_line = lines.next();
        let width = first_line.unwrap().split_ascii_whitespace().count();

        let all_same_length = lines.all(|ref l| l.split_ascii_whitespace().count() == width);
        assert!(all_same_length);

        let mut v = vec![];
        for line in encoding.lines() {
            let mut configuration = Vec::<u8>::new();
            for label in line.split_ascii_whitespace() {
                let value = if label_map.contains_key(label) {
                    label_map.get(label).unwrap().clone()
                } else {
                    let new_value = label_map.len() as u8;
                    label_map.insert(String::from(label), new_value);
                    new_value
                };

                configuration.push(value)
            }
            v.push(configuration);
        }

        Ok(Configurations { data: v })
    }

    pub fn from_configuration_data(
        configuration_data: Vec<Vec<u8>>,
    ) -> Result<Self, Box<dyn Error>> {
        assert!(!configuration_data.is_empty());
        assert!(!configuration_data[0].is_empty());

        let width = configuration_data[0].len();
        let all_same_length = configuration_data.iter().all(|l| l.len() == width);
        assert!(all_same_length);

        Ok(Configurations {
            data: configuration_data,
        })
    }

    /// Returns the count of labels in a configuration.
    pub fn get_labels_per_configuration(&self) -> usize {
        self.data[0].len()
    }

    /// Returns the count of labels in a configuration.
    pub fn get_configuration_count(&self) -> usize {
        self.data.len()
    }

    /// Returns configurations at `index`.
    pub fn get_configuration(&self, index: usize) -> &[u8] {
        assert!(index < self.get_configuration_count());
        &self.data[index]
    }

    /// Returns reference to configurations.
    pub fn get_configurations(&self) -> &Vec<Vec<u8>> {
        &self.data
    }

    /// Returns mutable reference to configurations.
    pub fn get_configuration_mut(&mut self) -> &mut Vec<Vec<u8>> {
        &mut self.data
    }

    /// Returns all unique permutations of labels, in each configuration.
    ///
    /// # Example
    /// Let Active configurations be
    /// ```text
    ///   A A B
    ///   A B C
    /// ```
    /// All permutations of those configurations are:
    /// ```text
    ///   A A B
    ///   A B A
    ///   B A A
    ///
    ///   A B C
    ///   A C B
    ///   B A C
    ///   B C A
    ///   C A B
    ///   C B A
    /// ```
    ///
    /// # Example
    /// ```
    /// use std::collections::HashMap;
    /// # use thesis_tool_lib::Configurations;
    /// let mut label_map = HashMap::<String, u8>::new();
    /// let configurations = Configurations::from_string("A B C", &mut label_map).unwrap();
    /// let permutations = configurations.get_permutations();
    /// let correct = vec![
    ///     vec![0, 1, 2],
    ///     vec![0, 2, 1],
    ///     vec![1, 0, 2],
    ///     vec![1, 2, 0],
    ///     vec![2, 0, 1],
    ///     vec![2, 1, 0]];
    /// assert_eq!(permutations, correct);
    /// ```
    pub fn get_permutations(&self) -> Vec<Vec<u8>> {
        self.data
            .iter()
            .map(|x| {
                let k = x.len();
                x.iter().map(|x| *x).permutations(k).unique().collect_vec()
            })
            .flatten()
            .collect_vec()
    }

    pub fn map_labels(&self, permutation: &Vec<u8>) -> Configurations {
        assert!(!permutation.is_empty());
        let data = self
            .data
            .iter()
            .map(|configuration| {
                configuration
                    .iter()
                    .map(|label| permutation[*label as usize])
                    .collect_vec()
            })
            .collect_vec();
        Configurations { data, ..*self }
    }

    pub fn sort(&mut self) {
        self.sort_labels_inside_configuration();
        self.sort_configurations();
    }

    fn sort_configurations(&mut self) {
        self.data.sort();
    }

    fn sort_labels_inside_configuration(&mut self) {
        self.data.iter_mut().for_each(|c| c.sort());
    }

    pub fn get_labels(&self) -> Vec<u8> {
        self.data.iter().flatten().copied().unique().collect_vec()
    }

    pub fn get_labels_set(&self) -> HashSet<u8> {
        HashSet::from_iter(self.data.iter().flatten().copied())
    }

    pub fn remove_configurations_containing_label(&mut self, labels: &[u8]) {
        self.data.retain(|configuration| {
            for label in labels {
                if configuration.contains(label) {
                    return false;
                }
            }
            true
        });
    }

    /// Generate powerset of configurations with specified degree and alphabet.
    pub fn generate_powerset(degree: usize, alphabet_length: u8) -> Vec<Configurations> {
        let alphabet = (0..alphabet_length).collect_vec();
        let powerset_of_labels = Self::generate_with_all_combinations(degree, &alphabet);

        let powerset_of_configurations = (1..=powerset_of_labels.get_configuration_count())
            .flat_map(|max_configurations| {
                powerset_of_labels
                    .get_configurations()
                    .iter()
                    .cloned()
                    .combinations(max_configurations)
            })
            .map(|data| Configurations::from_configuration_data(data).unwrap())
            .collect_vec();
        return powerset_of_configurations;
    }

    /// Generates `Configurations` that contains all combinations of the labels in `alphabet`.
    fn generate_with_all_combinations(degree: usize, alphabet: &Vec<u8>) -> Configurations {
        let data = alphabet
            .iter()
            .cloned()
            .combinations_with_replacement(degree)
            .collect_vec();
        Configurations::from_configuration_data(data).unwrap()
    }
}

impl PartialEq for Configurations {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl PartialOrd for Configurations {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        let mut label_map = HashMap::new();
        label_map.insert("A".to_string(), 0u8);
        label_map.insert("B".to_string(), 1u8);
        label_map.insert("C".to_string(), 2u8);

        let c0 = Configurations::from_string("A B B\nC C C", &mut label_map).unwrap();
        let c1 = Configurations::from_string("A B\nB C\nC C", &mut label_map).unwrap();
        let c2 = Configurations::from_string("A B\nB C\nC C", &mut label_map).unwrap();

        assert_ne!(c0, c1);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_sort() {
        let mut label_map = HashMap::new();
        label_map.insert("M".to_string(), 0u8);
        label_map.insert("U".to_string(), 1u8);
        label_map.insert("P".to_string(), 2u8);

        let mut c0 = Configurations::from_string("M U U\nP P P", &mut label_map).unwrap();
        let mut c1 = Configurations::from_string("U M U\nP P P", &mut label_map).unwrap();
        let mut c2 = Configurations::from_string("P P P\nU U M", &mut label_map).unwrap();
        let mut c3 = Configurations::from_string("M P P\nU U M", &mut label_map).unwrap();

        // Different configurations at first.
        assert_ne!(c0, c1);
        assert_ne!(c0, c2);
        assert_ne!(c1, c2);
        assert_ne!(c3, c0);
        assert_ne!(c3, c1);
        assert_ne!(c3, c2);

        // Sort all.
        c0.sort();
        c1.sort();
        c2.sort();
        c3.sort();

        // After sorting these are same.
        assert_eq!(c0, c1);
        assert_eq!(c1, c2);

        // After sorting these are still different.
        assert_ne!(c3, c0);
        assert_ne!(c3, c1);
        assert_ne!(c3, c2);
    }
}
