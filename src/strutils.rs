use itertools::Itertools;
use unicode_normalization::UnicodeNormalization;

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
        let mut sizes = Vec::with_capacity(string.len() + 1);

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

    fn char_at(&self, idx: usize) -> &str {
        if idx == self.len {
            return &self.string[self.char_byte_indices[idx]..];
        }
        self.substring(idx, idx + 1)
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

    fn suffix_array(&self) -> Vec<usize> {
        (0..self.len)
            .sorted_by_key(|i| self.substring(*i, self.len))
            .collect_vec()
    }

    fn lcp_array(&self, suffix_array: &[usize]) -> Vec<usize> {
        let mut lcp_array = Vec::with_capacity(suffix_array.len());
        lcp_array.push(0);
        for i in 0..suffix_array.len() {
            if i > 0 {
                let lcp = self.longest_common_prefix(suffix_array[i - 1], suffix_array[i]);
                lcp_array.push(lcp);
            }
        }
        lcp_array
    }

    /// Returns the _length_ of the longest common prefix of self.string[start_1..] and self.string[start_2..]
    fn longest_common_prefix(&self, start_1: usize, start_2: usize) -> usize {
        let mut char_count = 0;
        loop {
            let char_1 = self.char_at(start_1 + char_count);
            let char_2 = self.char_at(start_2 + char_count);
            if char_1 != char_2 {
                return char_count;
            }
            if char_count == self.len {
                return start_1;
            }
            char_count += 1;
        }
    }

    pub fn generate_factors(&self) -> impl Iterator<Item = &str> {
        let suffix_array = self.suffix_array();
        let lcp_array = self.lcp_array(&suffix_array);
        let mut i = 0;
        let mut j = 0;
        let mut first = true;
        std::iter::from_fn(move || {
            let suffix_len = self.len - suffix_array[i];
            if first {
                first = false;
                j = lcp_array[i];
                return Some("");
            }
            if j >= suffix_len {
                i += 1;
                if i >= suffix_array.len() {
                    return None;
                }
                j = lcp_array[i];
            }

            let start_pos = suffix_array[i];
            let end_pos = suffix_array[i] + j + 1;
            j += 1;
            let val = Some(self.substring(start_pos, end_pos));

            return val;
        })
    }

    pub fn count_factors(&self) -> usize {
        let mut counter = 0;
        let mut iter = self.generate_factors();
        while iter.next().is_some() {
            counter += 1;
        }
        counter
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

pub fn count_chars(string: &str) -> usize {
    let operator = CharOperator::new(string);
    operator.len
}

#[inline]
pub fn unicode_normalise(string: &str) -> String {
    string.nfc().collect()
}
