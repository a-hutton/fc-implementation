use crate::formula::EquationContent;
use crate::Substitution;
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

    fn multi_substring(&self, indices: &[usize]) -> Vec<&'a str> {
        let mut substrings = Vec::with_capacity(indices.len());
        let mut prev_index = 0;
        for &idx in indices {
            substrings.push(self.substring(prev_index, idx));
            prev_index = idx;
        }
        substrings
    }

    pub fn assign_by_partitions(
        &self,
        pattern: &'a Vec<EquationContent>,
        partitions: &[usize],
    ) -> Option<Substitution<'a>> {
        if pattern.len() != partitions.len() + 1 {
            return None;
        }
        let partitioned = self.multi_substring(partitions);
        let mut sub = Substitution::new();
        for (i, p) in pattern.iter().enumerate() {
            match p {
                EquationContent::Variable(var) => {
                    if sub.contains_key(var) && sub[var] != partitioned[i] {
                        return None;
                    }
                    sub.insert(var, partitioned[i]);
                }
                EquationContent::Constant(val) => {
                    if partitioned[i] != *val {
                        return None;
                    }
                }
            }
        }
        Some(sub)
    }

    pub fn len(&self) -> usize {
        self.len
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

pub fn slice_by_chars(original: &str, start: usize, end: usize) -> &str {
    if end < start {
        panic!("end > start");
    } else if end == start {
        return &original[start..end];
    }

    let bytes = original.as_bytes();
    println!("{:x?}", bytes);
    let mut char_count = 0;
    let mut start_byte_index = 0;
    let mut end_byte_index = 0;

    let mut cur_byte_index = 0;
    loop {
        let b = bytes[cur_byte_index];
        if char_count == start {
            start_byte_index = cur_byte_index;
        }

        // Character contains one byte
        if b & 0b10000000 == 0 {
            char_count += 1;
            cur_byte_index += 1;
        }
        // Character contains four bytes
        else if b & 0b11110000 == 0b11110000 {
            char_count += 1;
            cur_byte_index += 4;
        }
        // Character contains three bytes
        else if b & 0b11100000 == 0b11100000 {
            char_count += 1;
            cur_byte_index += 3;
        }
        // Character contains two bytes
        else if b & 0b11000000 == 0b11000000 {
            char_count += 1;
            cur_byte_index += 2;
        } else {
            panic!("Invalid byte index");
        }
        if char_count == end {
            end_byte_index = cur_byte_index;
            break;
        }
    }

    &original[start_byte_index..end_byte_index]
}

#[test]
fn test_slice_by_chars() {
    // The basics
    assert_eq!(slice_by_chars("abc", 0, 2), "ab");
    assert_eq!(slice_by_chars("abc", 0, 3), "abc");
    assert_eq!(slice_by_chars("abc", 0, 0), "");

    assert_eq!(slice_by_chars("abcdéf", 3, 5), "dé");

    assert_eq!(slice_by_chars("a🦩a", 1, 2), "🦩");
}
