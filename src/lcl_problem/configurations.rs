use std::{collections::HashMap, error::Error};

#[derive(Debug)]
pub struct Configurations {
    pub data: Vec<u8>,
    pub size: (usize, usize),
}

impl Configurations {
    pub fn from_str(
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
            size: (width, height),
        })
    }
}