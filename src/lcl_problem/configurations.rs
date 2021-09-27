use std::{collections::HashMap, error::Error};

use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct Configurations {
    data: Vec<u8>,
    labels_per_configuration: usize,
    configuration_count: usize,
}

impl Configurations {
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
