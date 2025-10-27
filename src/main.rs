mod equation_parser;
mod equations;
mod tests;

use crate::equations::WordEquation;
use itertools::Itertools;
use std::collections::HashMap;

fn main() {
    // find_solutions(r#"(x=y "b" && ¬y="")"#, "aab");
    let _ = find_solutions(
        &*equation_parser::parse_word_equation(r#"(x=y "b" && ¬y="")"#).unwrap(),
        "aab",
    );
}

fn find_solutions<'a>(equation: &'a dyn WordEquation, word: &'a str) -> Vec<Substitution<'a>> {
    let free_vars = equation.free_vars();
    let all_subs = all_possible_substitutions(free_vars, word);
    all_subs
        .iter()
        .filter(|sub| equation.check_substitution(sub))
        .cloned()
        .collect()
}

fn print_solution(sub: &Substitution) {
    let mut keys = sub.keys().collect::<Vec<_>>();
    keys.sort();
    // guaranteed to be the longest variable, helps for printing as a 'table'
    let universe_len = sub["U"].len();
    for (i, key) in keys.iter().enumerate() {
        let val = if sub[**key].is_empty() {
            "ε"
        } else {
            sub[**key]
        };
        print!("{}: {:width$}", key, val, width = universe_len);
        if i < keys.len() - 1 {
            print!(" | ")
        }
    }

    println!()
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
            // Unique substrings only
            if !factors.contains(&substr) {
                factors.push(substr);
            }
        }
    }
    factors.push(w);
    factors
}

type Substitution<'a> = HashMap<&'a str, &'a str>;

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
