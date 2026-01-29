mod formula;
mod formula_parser;
mod strutils;
mod tests;

use crate::strutils::CharOperator;
use crate::formula::{Formula, UNIVERSE_CONSTANT};
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

    let text_chars = CharOperator::new(text.as_str());

    match args.command {
        ProgramCommand::GenerateFactors => {
            // Generate factors only
            let factors = text_chars.generate_factors();
            if args.quiet {
                println!("Generated {} Factors", factors.len());
            } else {
                for factor in factors {
                    println!("{}", factor);
                }
            }
        }
        ProgramCommand::FindSolutions => {
            let formula_str = args.pattern.unwrap();
            let parsed_formula = formula_parser::parse_formula_str(formula_str.as_str());
            match parsed_formula {
                None => {
                    println!("Failed to parse formula, exiting");
                }
                Some(parsed_formula) => {
                    let solutions = find_solutions(&*parsed_formula, &text_chars);
                    println!("Found {} solutions", solutions.len());
                    if !args.quiet {
                        print_solutions(&solutions, text.as_str());
                    }
                }
            }
        }
        ProgramCommand::CheckAssignment => {
            let mut substitution = Substitution::new();
            let assignment_strings = &args.assignment.unwrap();
            for var_assignment in assignment_strings {
                // Split at first colon
                if let Some((var_name, val)) = var_assignment.split_once(":") {
                    substitution.insert(var_name, val);
                } else {
                    println!(
                        "Invalid assignment string format: '{}'. Expected <VAR_NAME>:<VALUE>",
                        var_assignment
                    );
                    return;
                }
            }

            let formula_str = args.pattern.unwrap();
            let parsed_formula = formula_parser::parse_formula_str(formula_str.as_str());
            match parsed_formula {
                None => {
                    println!("Failed to parse formula, exiting");
                }
                Some(parsed_formula) => {
                    let free_vars = parsed_formula.free_vars();
                    for free_var in &free_vars {
                        if !substitution.contains_key(free_var) {
                            println!(
                                "Free variable {} from formula is missing from assignment",
                                free_var
                            );
                            return;
                        }
                    }
                    for var_name in substitution.keys() {
                        if !free_vars.contains(var_name) {
                            println!("Unknown variable {} given assignment", var_name);
                            return;
                        }
                    }
                    let factors = generate_factors(text.as_str());

                    // TODO - is this desired/necessary
                    substitution.insert(UNIVERSE_CONSTANT, text.as_str());

                    let is_satisfying = parsed_formula.check_substitution(&substitution, &factors);
                    println!("Is Satisfying Assignment: {}", is_satisfying);
                    if !args.quiet {
                        for (var, val) in &substitution {
                            println!("{}: \"{}\"", var, val)
                        }
                    }
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
    #[arg(
        short,
        long,
        required_if_eq("command", "all-solutions"),
        required_if_eq("command", "check-assignment")
    )]
    pattern: Option<String>,
    #[command(flatten)]
    /// Flatten means this option will be transparent to user
    text_option: TextOption,
    /// When set to quiet, the full table of found solutions won't be printed, just its size
    #[arg(short, long)]
    quiet: bool,
    /// Options for testing individual components of the program
    #[arg(short, long, value_enum, default_value_t = ProgramCommand::FindSolutions)]
    command: ProgramCommand,
    /// An assignment to be verified (if command = check-assignment). Formatted as '<VAR_NAME>:<ASSIGNMENT_STRING>'
    #[arg(short, long, required_if_eq("command", "check-assignment"))]
    assignment: Option<Vec<String>>,
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
    /// Find solutions for the formula on the text
    FindSolutions,
    /// Print all factors of the text
    GenerateFactors,
    /// Check a given assignment of variables to values holds
    CheckAssignment,
}

/// Find all the assignments to variables in a formula based on values in the universe of
/// substrings of `word` that satisfy the given formula.
fn find_solutions<'a>(formula: &'a dyn Formula, word: &'a CharOperator) -> Vec<Substitution<'a>> {
    let free_vars = formula.free_vars();
    let universe = word.generate_factors();
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

/// Represents a substitution (σ in the literature). An assignment mapping variable names to values
/// from the universe
type Substitution<'a> = HashMap<&'a str, &'a str>;

/// Constructs all possible assignments from a universe of factors of the universe constant `w`
/// to a given list of variables. Works by brute force over the Cartesian product repeated _n_ times
/// for _n_ variables
fn all_possible_substitutions<'a>(
    var_names: Vec<&'a str>,
    w: &'a CharOperator,
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
            sub.insert(formula::UNIVERSE_CONSTANT, w.as_str());
            Some(sub)
        } else {
            None
        }
    })
}
