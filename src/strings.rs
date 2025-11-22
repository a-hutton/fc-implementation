use std::cmp::min;
use std::str::from_utf8;
use suffix::SuffixTable;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

fn create_suffix_array(word: &str) -> Vec<usize> {
    let graphemes = UnicodeSegmentation::graphemes(word, true).collect::<Vec<&str>>();
    let mut suffix_array: Vec<usize> = Vec::with_capacity(graphemes.len());
    let mut byte_count = word.len();
    for r in graphemes.iter().rev() {
        byte_count -= r.len();
        suffix_array.push(byte_count);
    }
    suffix_array.sort_by_key(|&byte| word[byte..].to_lowercase());

    suffix_array
}

fn create_lcp_array(suffix_array: &[usize], word: &str) -> Vec<usize> {
    let graphemes = UnicodeSegmentation::graphemes(word, true).collect::<Vec<&str>>();
    let mut lcp_arr = Vec::with_capacity(suffix_array.len());
    lcp_arr.push(0);
    let mut grapheme_index = 0;
    for i in 0..(suffix_array.len() - 1) {
        let a = &word[suffix_array[i]..];
        let b = &word[suffix_array[i + 1]..];
        let mut lcp = 0;
        grapheme_index = i;
        loop {
            let shorter_size = min(a.len(), b.len());
            // a and b are different
            // the 'next' char in a and b could be ASCII, or any width
            // if rust tries to slice n bytes into one &str and it's fine, but the other it's not
            // then it will panic. We know from this that a[..lcp + next_char_width]!=b[..lcp+next_char_width]
            if lcp < shorter_size {
                if grapheme_index >= suffix_array.len() {
                    break;
                }
                let next_char_width = graphemes[grapheme_index].len();
                let a_prefix = from_utf8(&a.as_bytes()[..lcp + next_char_width]);
                let b_prefix = from_utf8(&b.as_bytes()[..lcp + next_char_width]);
                if a_prefix.is_err() || b_prefix.is_err() {
                    break;
                }
                if a[..lcp + next_char_width] == b[..lcp + next_char_width] {
                    lcp += next_char_width;
                    grapheme_index += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        lcp_arr.push(lcp);
    }
    lcp_arr
}

#[test]
fn test_lcp() {
    let w = "bañaña";
    // let w = "banana";
    // let w = "AïaA";
    let w = &w.nfc().collect::<String>();
    // let w = "WZYaZ";
    let x = create_suffix_array(w);
    for &s in &x {
        println!("{} {}", s, &w[s..]);
    }
    println!("-----");
    let y = create_lcp_array(&x, w);
    println!("lcp: {:?}", y)
}

/// Constructs a list of all substrings of a given word. This assumes that the empty string is a
/// factor of all words
pub fn generate_factors<'a>(word: &'a str, suffix_table: &'a SuffixTable) -> Vec<&'a str> {
    println!("{:?}", suffix_table);
    let lcp_array = suffix_table.lcp_lens();
    // n = number of 'characters' in string
    let n = suffix_table.len();
    let mut factors = Vec::with_capacity(n * (n + 1) / 2);
    for i in 0..(n) {
        let start = suffix_table.suffix(i);
        let lcp = lcp_array[i];
        if start.len() >= n {
            continue;
        }
        for length in (lcp) as usize..(word.len() - start.len()) {
            let factor = from_utf8(&start[..length].as_bytes());
            if factor.is_err() {
                println!("utf8 error at {}..{}", start, length);
                continue;
            }
            factors.push(factor.unwrap());
        }
    }
    factors.push("");
    factors
}

#[cfg(test)]
#[test]
fn test_generate_factors() {
    struct TestCase {
        word: &'static str,
        num_factors: usize,
    }
    let cases = vec![
        TestCase {
            word: "banana",
            num_factors: 16, // ε a an ana anan anana b ba ban bana banan banana n na nan nana
        },
        TestCase {
            word: "Banana",
            num_factors: 16, // ε a an ana anan anana B Ba Ban Bana Banan Banana n na nan nana
        },
        TestCase {
            word: "ababa",
            num_factors: 10, // ε a ab aba abab ababa b ba bab baba
        },
        TestCase {
            word: "añaña",
            num_factors: 10, // ε a añ aña añañ añaña ñ ña ñañ ñaña
        },
    ];
    for case in cases {
        let suffix_table = SuffixTable::new(case.word);
        let factors = generate_factors(case.word, &suffix_table);
        println!("Factors: {:?}", factors);
        assert_eq!(factors.len(), case.num_factors);
    }
}
