use std::{collections::HashMap, error::Error};

use itertools::Itertools;

/// A container for set of configurations that are used to define an LCL problem.
///
/// A configuration is a multiset of labels.
/// A new Configuration can be created by using method [`Configurations::new`].
///
/// Contained configurations can be accessed with different methods.
/// It is also possible to access all unique permutations of each configuration with [`Configurations::get_permutations`].
#[derive(Debug, Clone)]
pub struct Configurations {
    data: Vec<u8>,
    labels_per_configuration: usize,
    configuration_count: usize,
}

impl Configurations {
    /// Creates Configuration instance from given `encoding` using given `symbol_map`.
    ///
    /// Encoding is formated as a multirow string where each configuration is separated with linebreak and each label is separated with space.
    /// Each configuration has to be equally long.
    ///
    /// Internally each symbol is mapped to unsigned integers and then saved in vector as `u8`.
    /// By default, symbols increase starting from 0.
    /// A symbol_map is supposed to be given if it is desired to have multiple [`Configurations`] instances using same mapping of symbols.
    ///
    ///
    /// # Example
    /// ```
    /// use std::collections::HashMap;
    /// # use thesis_tool_lib::Configurations;
    /// let mut symbol_map = HashMap::<String, u8>::new();
    /// let configurations = Configurations::new("A B C\nA A B\nC C C", &mut symbol_map).unwrap();
    /// ```
    pub fn new(
        encoding: &str,
        symbol_map: &mut HashMap<String, u8>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut lines = encoding.lines();
        let first_line = lines.next();
        let width = first_line.unwrap().split_ascii_whitespace().count();

        let all_same_length = lines.all(|ref l| l.split_ascii_whitespace().count() == width);
        assert!(all_same_length);

        let mut v = Vec::<u8>::new();
        for line in encoding.lines() {
            for symbol in line.split_ascii_whitespace() {
                let value = if symbol_map.contains_key(symbol) {
                    symbol_map.get(symbol).unwrap().clone()
                } else {
                    let new_value = symbol_map.len() as u8;
                    symbol_map.insert(String::from(symbol), new_value);
                    new_value
                };

                v.push(value)
            }
        }

        let height = encoding.lines().count();

        Ok(Configurations {
            data: v,
            labels_per_configuration: width,
            configuration_count: height,
        })
    }

    /// Returns the count of labels in a configuration.
    pub fn get_labels_per_configuration(&self) -> usize {
        self.labels_per_configuration
    }

    /// Returns the count of labels in a configuration.
    pub fn get_configuration_count(&self) -> usize {
        self.configuration_count
    }

    /// Returns configurations at `index`.
    pub fn get_configuration(&self, index: usize) -> &[u8] {
        assert!(index < self.configuration_count);
        let begin = self.labels_per_configuration * index;
        let end = begin + self.labels_per_configuration;
        &self.data[begin..end]
    }

    /// Returns configurations as chunks of labels.
    pub fn get_configurations_chunks(&self) -> itertools::IntoChunks<std::slice::Iter<u8>> {
        self.data.iter().chunks(self.labels_per_configuration)
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
    /// let mut symbol_map = HashMap::<String, u8>::new();
    /// let configurations = Configurations::new("A B C", &mut symbol_map).unwrap();
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
        let configurations_chunks = self.get_configurations_chunks();
        let configurations_vec = configurations_chunks.into_iter().map(|x| x.collect_vec());
        configurations_vec
            .map(|x| {
                let k = x.len();
                x.iter().map(|x| **x).permutations(k).unique().collect_vec()
            })
            .flatten()
            .collect_vec()
    }
}
