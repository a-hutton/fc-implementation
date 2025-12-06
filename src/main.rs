mod formula;
mod formula_parser;
mod tests;

use crate::formula::Formula;
use clap::Parser;
use itertools::Itertools;
use std::collections::HashMap;
use std::fs;
use unicode_segmentation::UnicodeSegmentation;

fn main() {
    let args = Args::parse();

    let parsed_formula = formula_parser::parse_formula_str(args.pattern.as_str());
    if parsed_formula.is_none() {
        println!("Failed to parse formula, exiting");
        return;
    }
    let parsed_formula = parsed_formula.unwrap();

    let content: String;
    let text = if args.file {
        content = fs::read_to_string(args.text.as_str()).unwrap();
        content.as_str()
    } else {
        args.text.as_str()
    };
    let factors = generate_factors(text);
    let solutions = find_solutions(&*parsed_formula, text, &factors);
    if !args.quiet {
        print_solutions(solutions, args.text.as_str(), parsed_formula.free_vars());
    } else {
        let count = solutions.fold(0, |x, _| x + 1);
        println!("Found {} solutions", count);
    }
}

#[derive(clap::Parser)]
#[command(version)]
struct Args {
    /// The formula used to search the text
    pattern: String,
    /// The text universe the search is applied to
    text: String,
    /// Is the argument [`text`] a text filename
    #[arg(short, long)]
    file: bool,
    /// When set to quiet, the full table of found solutions won't be printed, just its size
    #[arg(short, long)]
    quiet: bool,
}

/// Find all the assignments to variables in a formula based on values in the universe of
/// substrings of `word` that satisfy the given formula.
fn find_solutions<'a>(
    formula: &'a dyn Formula,
    word: &'a str,
    universe: &[&'a str],
) -> impl Iterator<Item = Substitution<'a>> {
    let free_vars = formula.free_vars();
    let all_subs = all_possible_substitutions(free_vars, word, universe);
    all_subs
        .filter(move |sub| formula.check_substitution(sub, universe))
        .peekable()
}

/// Prints to stdout a pretty-printed CSV formatted table of all substitutions
fn print_solutions<'a>(
    subs: impl Iterator<Item = Substitution<'a>>,
    universe: &str,
    mut var_names: Vec<&'a str>,
) -> usize {
    let mut peekable = subs.peekable();
    if peekable.peek().is_none() {
        println!("No substitutions found");
        return 0;
    }

    var_names.sort();
    // guaranteed to be the longest variable, helps for printing as a 'table'
    let universe_len = UnicodeSegmentation::graphemes(universe, true).count();

    // print var names
    for (i, var_name) in var_names.iter().enumerate() {
        print!("{var:width$}", var = var_name, width = universe_len);
        if i < var_names.len() - 1 {
            print!(", ")
        }
    }
    println!();

    let mut count = 0;
    for sub in peekable {
        for (i, key) in var_names.iter().enumerate() {
            let val = if sub[*key].is_empty() {
                "ε"
            } else {
                sub[*key]
            };
            print!("{val:width$}", val = val, width = universe_len);
            if i < var_names.len() - 1 {
                print!(", ")
            }
        }
        println!();
        count += 1;
    }
    count
}

/// Constructs a list of all substrings of a given word. This assumes that the empty string is a
/// factor of all words
fn generate_factors(w: &str) -> Vec<&str> {
    // maximum possible number of substrings: n(n+1)/2
    let num_factors = w.len() * (w.len() + 1) / 2 + 1;
    let mut factors = Vec::with_capacity(num_factors);
    factors.push("");

    let unicode_chars = UnicodeSegmentation::graphemes(w, true).collect::<Vec<&str>>();

    // sliding window for each possible length of subword
    for len in 1..unicode_chars.len() {
        for i in 0..(unicode_chars.len() - len + 1) {
            let start_byte_offset = unicode_chars[..i].iter().map(|b| b.len()).sum::<usize>();
            let end_byte_offset = unicode_chars[..i + len]
                .iter()
                .map(|b| b.len())
                .sum::<usize>();
            let substr = &w[start_byte_offset..end_byte_offset];
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
fn all_possible_substitutions<'a>(
    var_names: Vec<&'a str>,
    w: &'a str,
    universe: &[&'a str],
) -> impl Iterator<Item = Substitution<'a>> {
    let var_vals_iter = std::iter::repeat_n(universe, var_names.len());
    let mut subs_iter = var_vals_iter.multi_cartesian_product();
    std::iter::from_fn(move || {
        if let Some(sub_vals) = subs_iter.next() {
            let mut sub = Substitution::new();
            for i in 0..sub_vals.len() {
                sub.insert(var_names[i], sub_vals[i]);
            }
            sub.insert(formula::UNIVERSE_CONSTANT, w);
            Some(sub)
        } else {
            None
        }
    })
}
