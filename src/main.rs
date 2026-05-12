mod assignment;
mod formula;
mod formula_parser;
mod strutils;
mod tests;

use crate::assignment::Assignment;
use crate::formula::Formula;
use crate::strutils::{unicode_normalise, CharOperator};
use clap::Parser;
use std::fs;

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
    let text = unicode_normalise(&text);
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
                    let solutions = find_solutions(&parsed_formula, &text_chars);
                    println!("Found {} solutions", solutions.len());
                    if !args.quiet {
                        print_assignments(&solutions, text.as_str());
                    }
                }
            }
        }
        ProgramCommand::CheckAssignment => {
            let assignment_strings = &args.assignment.unwrap();
            let mut assignment = Assignment::new(assignment_strings.len(), text_chars.as_str());
            for var_assignment in assignment_strings {
                // Split at first colon
                if let Some((var_name, val)) = var_assignment.split_once(":") {
                    assignment.insert(var_name, val);
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
                        if !assignment.keys.contains(free_var) {
                            println!(
                                "Free variable {} from formula is missing from assignment",
                                free_var
                            );
                            return;
                        }
                    }
                    for var_name in &assignment.keys {
                        if !free_vars.contains(var_name) {
                            println!("Unknown variable {} given assignment", var_name);
                            return;
                        }
                    }
                    let is_satisfying =
                        parsed_formula.is_satisfying_assignment(&text_chars, &assignment);
                    println!("Is Satisfying Assignment: {}", is_satisfying);
                    if !args.quiet {
                        for i in 0..assignment.keys.len() {
                            println!("{}: \"{}\"", assignment.keys[i], assignment.values[i]);
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
// Uses clap to require at least one of these, but no more than one.
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
fn find_solutions<'a>(formula: &'a Formula, word: &'a CharOperator) -> Vec<Assignment<'a>> {
    formula.all_solutions(word)
}

/// Prints to stdout a pretty-printed CSV formatted table of all given assignments
fn print_assignments(assignments: &Vec<Assignment>, universe: &str) -> usize {
    if assignments.is_empty() {
        println!("No satisfying assignments found");
        return 0;
    }

    let max_width = 80; // max column width

    // guaranteed to be the longest variable, helps for printing as a 'table'
    let universe_len = strutils::count_chars(universe);

    let var_names = &assignments[0].keys;
    // print var names
    for (i, var_name) in var_names.iter().enumerate() {
        print!("{var:width$}", var = var_name, width = max_width);
        if i < var_names.len() - 1 {
            print!(", ")
        }
    }
    println!();

    let mut count = 0;
    for assignment in assignments {
        for i in 0..assignment.keys.len() {
            let val = if assignment.values[i].is_empty() {
                CharOperator::new("ε")
            } else {
                CharOperator::new(assignment.values[i])
            };
            let val = if val.len() > max_width - 3 {
                String::from(val.substring(0, max_width - 3)) + "..."
            } else {
                String::from(val.as_str())
            };
            print!("{val:width$}", val = val, width = max_width);
            if i < var_names.len() - 1 {
                print!(", ")
            }
        }
        println!();
        count += 1;
    }
    count
}
