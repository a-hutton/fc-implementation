use crate::generate_factors;
use itertools::Itertools;

pub struct CharOperator<'a> {
    string: &'a str,
    char_byte_indices: Vec<usize>,
    len: usize,
}

impl<'a> CharOperator<'a> {
    pub fn new(string: &'a str) -> CharOperator<'a> {
        if string.is_empty() {
            return CharOperator {
                string,
                char_byte_indices: vec![],
                len: 0,
            };
        };

        let bytes = string.as_bytes();
        let mut sizes = Vec::with_capacity(string.len());

        let mut char_count = 0;

        let mut cur_byte_index = 0;
        loop {
            let b = bytes[cur_byte_index];

            // Character contains one byte
            if b & 0b10000000 == 0 {
                char_count += 1;
                sizes.push(cur_byte_index);
                cur_byte_index += 1;
            }
            // Character contains four bytes
            else if b & 0b11110000 == 0b11110000 {
                char_count += 1;
                sizes.push(cur_byte_index);
                cur_byte_index += 4;
            }
            // Character contains three bytes
            else if b & 0b11100000 == 0b11100000 {
                char_count += 1;
                sizes.push(cur_byte_index);
                cur_byte_index += 3;
            }
            // Character contains two bytes
            else if b & 0b11000000 == 0b11000000 {
                char_count += 1;
                sizes.push(cur_byte_index);
                cur_byte_index += 2;
            } else {
                panic!("Invalid byte index");
            }

            if cur_byte_index >= string.len() {
                break;
            }
        }
        sizes.push(string.len());

        CharOperator {
            string,
            char_byte_indices: sizes,
            len: char_count,
        }
    }

    pub fn substring(&self, start: usize, end: usize) -> &'a str {
        if start == end {
            return &self.string[start..end];
        }
        if start > end || end > self.len {
            panic!(
                "Invalid substring indices for start: {}, end: {}, len: {}",
                start, end, self.len
            );
        }
        &self.string[self.char_byte_indices[start]..self.char_byte_indices[end]]
    }

    pub fn multi_substring(&self, positions: &[usize]) -> Vec<&'a str> {
        // TODO - should this be an iterator (iterfunc?)
        let mut substrings = Vec::with_capacity(positions.len());
        for i in 1..positions.len() {
            let start_index = self.char_byte_indices[positions[i - 1]];
            let end_index = self.char_byte_indices[positions[i]];
            substrings.push(&self.string[start_index..end_index]);
        }
        substrings
    }

    pub fn find(&self, needle: &str) -> Vec<usize> {
        let byte_matches = self
            .string
            .match_indices(needle)
            .map(|(i, _)| i)
            .collect_vec();
        if byte_matches.is_empty() {
            return vec![];
        }
        let mut char_matches = Vec::with_capacity(byte_matches.len());
        for (_, b) in byte_matches.iter().enumerate() {
            for (j, c) in self.char_byte_indices.iter().enumerate() {
                if *b == *c {
                    char_matches.push(j);
                    break;
                }
            }
        }
        char_matches
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_str(&self) -> &str {
        self.string
    }

    pub fn generate_factors(&self) -> Vec<&str> {
        generate_factors(self.string)
    }
}

#[test]
fn test_chars_substring() {
    // The basics
    assert_eq!(CharOperator::new("abc").substring(0, 2), "ab");
    assert_eq!(CharOperator::new("abc").substring(0, 3), "abc");
    assert_eq!(CharOperator::new("abc").substring(0, 0), "");

    assert_eq!(CharOperator::new("abcdéf").substring(3, 5), "dé");

    assert_eq!(CharOperator::new("a🦩a").substring(1, 2), "🦩");
}
