mod equations;
use itertools::Itertools;
use std::collections::HashMap;

fn main() {
    println!("Hello there")
}

/// Assumes that the empty string is a factor of all words
fn generate_factors(w: &str) -> Vec<&str> {
    // number of substrings: n(n+1)/2
    let num_factors = w.len() * (w.len() + 1) / 2 + 1;
    let mut factors = Vec::with_capacity(num_factors);
    factors.push("");
    // sliding window for each possible length of subword
    for len in 1..w.len() {
        for pos in 0..(w.len() - len + 1) {
            let substr = &w[pos..pos + len];
            factors.push(substr);
        }
    }
    factors.push(w);
    factors
}

type Substitution = HashMap<&'static str, &'static str>;

fn all_possible_substitutions(var_names: Vec<&'static str>, w: &'static str) -> Vec<Substitution> {
    let factors = generate_factors(w);

    let var_vals_iter = std::iter::repeat(factors).take(var_names.len());
    let subs_iter = var_vals_iter.multi_cartesian_product();
    let mut substitutions = Vec::with_capacity(subs_iter.try_len().unwrap());
    for sub_vals in subs_iter {
        let mut sub = Substitution::new();
        for i in 0..sub_vals.len() {
            sub.insert(&var_names[i], sub_vals[i]);
        }
        substitutions.push(sub);
    }
    substitutions
}

#[test]
fn test_all_subs() {
    let subs = all_possible_substitutions(vec!["x", "y", "z"], "abcd");
    println!("{:?}", subs);
    assert!(true);
}
