mod formula;
mod formula_parser;
mod tests;

use crate::formula::Formula;
use itertools::Itertools;
use std::collections::HashMap;
use std::env;

fn main() {
    let search_pattern = env::args().nth(1);
    let universe = env::args().nth(2);
    if search_pattern.is_none() || universe.is_none() {
        println!("Usage: cargo run -- <pattern> <text>");
        return;
    }
    let search_pattern = search_pattern.unwrap();
    let universe = universe.unwrap();

    let parsed_formula = formula_parser::parse_formula_str(search_pattern.as_str());
    if parsed_formula.is_none() {
        println!("Failed to parse formula, exiting");
        return;
    }
    let parsed_formula = parsed_formula.unwrap();
    let solutions = find_solutions(&*parsed_formula, universe.as_str());
    println!("Found {} solutions", solutions.len());
    print_solutions(&solutions, universe.as_str());
}

/// Find all the assignments to variables in a formula based on values in the universe of
/// substrings of `word` that satisfy the given formula.
fn find_solutions<'a>(formula: &'a dyn Formula, word: &'a str) -> Vec<Substitution<'a>> {
    let free_vars = formula.free_vars();
    let all_subs = all_possible_substitutions(free_vars, word);
    all_subs
        .iter()
        .filter(|sub| formula.check_substitution(sub))
        .cloned()
        .collect()
}

/// Prints to stdout a pretty-printed CSV formatted table of all substitutions
fn print_solutions(subs: &Vec<Substitution>, universe: &str) {
    if subs.is_empty() {
        println!("No solutions found");
        return;
    }
    let mut var_names = subs[0].keys().collect::<Vec<_>>();
    var_names.sort();
    // guaranteed to be the longest variable, helps for printing as a 'table'
    let universe_len = universe.len();

    // print var names
    for (i, var_name) in var_names.iter().enumerate() {
        print!("{var:width$}", var = var_name, width = universe_len);
        if i < var_names.len() - 1 {
            print!(", ")
        }
    }
    println!();

    for sub in subs {
        for (i, key) in var_names.iter().enumerate() {
            let val = if sub[**key].is_empty() {
                "ε"
            } else {
                sub[**key]
            };
            print!("{val:width$}", val = val, width = universe_len);
            if i < var_names.len() - 1 {
                print!(", ")
            }
        }
        println!()
    }
}

/// Constructs a list of all substrings of a given word. This assumes that the empty string is a
/// factor of all words
fn generate_factors(w: &str) -> Vec<&str> {
    // number of substrings: n(n+1)/2
    let num_factors = w.len() * (w.len() + 1) / 2 + 1;
    let mut factors = Vec::with_capacity(num_factors);
    factors.push("");
    // sliding window for each possible length of subword
    for len in 1..w.len() {
        for pos in 0..(w.len() - len + 1) {
            let substr = &w[pos..pos + len];
            // Unique substrings only
            if !factors.contains(&substr) {
                factors.push(substr);
            }
        }
    }
    factors.push(w);
    factors
}

/// Represents a substitution (σ in the literature). An assignment mapping variable names to values
/// from the universe
type Substitution<'a> = HashMap<&'a str, &'a str>;

/// Constructs all possible assignments from a universe of factors of the universe constant `w`
/// to a given list of variables. Works by brute force over the Cartesian product repeated _n_ times
/// for _n_ variables
fn all_possible_substitutions<'a>(var_names: Vec<&'a str>, w: &'a str) -> Vec<Substitution<'a>> {
    let factors = generate_factors(w);

    let var_vals_iter = std::iter::repeat_n(factors, var_names.len());
    let subs_iter = var_vals_iter.multi_cartesian_product();
    let mut substitutions = Vec::with_capacity(subs_iter.try_len().unwrap());
    for sub_vals in subs_iter {
        let mut sub = Substitution::new();
        for i in 0..sub_vals.len() {
            sub.insert(var_names[i], sub_vals[i]);
        }
        sub.insert("U", w);
        substitutions.push(sub);
    }
    substitutions
}
