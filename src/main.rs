mod formula;
mod formula_parser;
mod strutils;
mod tests;

use crate::formula::Formula;
use clap::Parser;
use itertools::Itertools;
use std::collections::HashMap;
use std::fs;
use unicode_segmentation::UnicodeSegmentation;

fn main() {
    let args = Args::parse();

    let text = if let Some(filename) = args.text_option.file {
        let res = fs::read_to_string(filename.as_str());
        if let Ok(file_content) = res {
            file_content
        } else {
            panic!("File '{}' not found", filename);
        }
    } else {
        args.text_option.text.unwrap()
    };

    match args.command {
        ProgramCommand::GenerateFactors => {
            // Generate factors only
            let factors = generate_factors(text.as_str());
            for factor in factors {
                println!("{}", factor);
            }
        }
        ProgramCommand::AllSolutions => {
            let formula_str = args.pattern.unwrap();
            let parsed_formula = formula_parser::parse_formula_str(formula_str.as_str());
            match parsed_formula {
                None => {
                    println!("Failed to parse formula, exiting");
                }
                Some(parsed_formula) => {
                    let text_chars = strutils::CharOperator::new(text);
                    let solutions = find_solutions(&*parsed_formula, text.as_str(), text_chars);
                    println!("Found {} solutions", solutions.len());
                    if !args.quiet {
                        print_solutions(&solutions, text.as_str());
                    }
                }
            }
        }

}

#[derive(clap::Parser)]
#[command(version)]
struct Args {
    /// The formula used to search the text
    // pattern not needed for factor generation
    #[arg(short, long, required_if_eq("command", "all-solutions"))]
    pattern: Option<String>,
    #[command(flatten)]
    /// Flatten means this option will be transparent to user
    text_option: TextOption,
    /// When set to quiet, the full table of found solutions won't be printed, just its size
    #[arg(short, long)]
    quiet: bool,
    /// Options for testing individual components of the program
    #[arg(short, long, value_enum, default_value_t = ProgramCommand::AllSolutions)]
    command: ProgramCommand,
}

#[derive(clap::Args, Clone, Debug)]
#[group(required = true, multiple = false)]
/// Uses clap to require at least one of these, but no more than one.
struct TextOption {
    /// The text universe the search is applied to. Mutually exclusive with --text
    #[arg(short, long, required_unless_present = "file")]
    text: Option<String>,
    /// Filename containing the text universe to search. Mutually exclusive with --file
    #[arg(short, long, required_unless_present = "text")]
    file: Option<String>,
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum ProgramCommand {
    /// Find all solutions for the formula on the text
    AllSolutions,
    /// Print all factors of the text
    GenerateFactors,
}

/// Find all the assignments to variables in a formula based on values in the universe of
/// substrings of `word` that satisfy the given formula.
fn find_solutions<'a>(
    formula: &'a dyn Formula,
    word: &'a str,
    word_chars: &'a strutils::CharOperator,
) -> Vec<Substitution<'a>> {
    let free_vars = formula.free_vars();
    let universe = word_chars.generate_factors();
    let all_subs = all_possible_substitutions(free_vars, word, &universe);
    all_subs
        .filter(|sub| formula.check_substitution(sub, &universe))
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
    let universe_len = UnicodeSegmentation::graphemes(universe, true).count();

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

#[test]
fn test() {
    let w = "“de”😂🇬🇧";
    let g = UnicodeSegmentation::graphemes(w, true).collect::<Vec<&str>>();
    for i in 0..(g.len() - 1) {
        let start_byte_offset = g[..i].iter().map(|b| b.len()).sum::<usize>();
        let end_byte_offset = g[..i + 2].iter().map(|b| b.len()).sum::<usize>();
        println!("{:?}", &w[start_byte_offset..end_byte_offset]);
    }
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
    universe: &Vec<&'a str>,
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
