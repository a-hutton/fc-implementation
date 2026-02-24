mod formula;
mod formula_parser;
mod strutils;
mod tests;

use crate::formula::{EquationContent, Formula, UNIVERSE_CONSTANT};
use crate::strutils::CharOperator;
use clap::Parser;
use itertools::Itertools;
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
                    let solutions = find_solutions(&parsed_formula, &text_chars);
                    println!("Found {} solutions", solutions.len());
                    if !args.quiet {
                        print_solutions(&solutions, text.as_str());
                    }
                }
            }
        }
        ProgramCommand::CheckAssignment => {
            let assignment_strings = &args.assignment.unwrap();
            let mut substitution = Substitution::new(assignment_strings.len());
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
                        if !substitution.keys.contains(free_var) {
                            println!(
                                "Free variable {} from formula is missing from assignment",
                                free_var
                            );
                            return;
                        }
                    }
                    for var_name in &substitution.keys {
                        if !free_vars.contains(var_name) {
                            println!("Unknown variable {} given assignment", var_name);
                            return;
                        }
                    }
                    let is_satisfying =
                        parsed_formula.check_substitution(&text_chars, &substitution);
                    println!("Is Satisfying Assignment: {}", is_satisfying);
                    if !args.quiet {
                        for i in 0..substitution.keys.len() {
                            println!("{}: \"{}\"", substitution.keys[i], substitution.values[i]);
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
fn find_solutions<'a>(formula: &'a Formula, word: &'a CharOperator) -> Vec<Substitution<'a>> {
    formula.all_solutions(word)
}

/// Prints to stdout a pretty-printed CSV formatted table of all substitutions
fn print_solutions(subs: &Vec<Substitution>, universe: &str) -> usize {
    if subs.is_empty() {
        println!("No satisfying assignments found");
        return 0;
    }

    // guaranteed to be the longest variable, helps for printing as a 'table'
    let universe_len = strutils::count_chars(universe);

    let var_names = &subs[0].keys;
    // print var names
    for (i, var_name) in var_names.iter().enumerate() {
        print!("{var:width$}", var = var_name, width = universe_len);
        if i < var_names.len() - 1 {
            print!(", ")
        }
    }
    println!();

    let mut count = 0;
    for sub in subs {
        for i in 0..sub.keys.len() {
            let val = if sub.values[i].is_empty() {
                "ε"
            } else {
                sub.values[i]
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
#[derive(Eq, PartialEq, Hash, Debug)]
struct Substitution<'a> {
    keys: Vec<&'a str>,
    values: Vec<&'a str>,
    universe_constant: &'a str,
}
impl<'a> Substitution<'a> {
    fn new(hint: usize) -> Self {
        Substitution {
            keys: Vec::with_capacity(hint),
            values: Vec::with_capacity(hint),
            universe_constant: "",
        }
    }

    fn from_vars(vars: &[&'a str], w: &'a str) -> Self {
        let len = vars.len();
        let mut keys = Vec::with_capacity(vars.len());
        for var in vars {
            if !keys.contains(var) {
                keys.push(var);
            }
        }

        Substitution {
            keys,
            values: Vec::with_capacity(len),
            universe_constant: w,
        }
    }

    fn join(a: &Substitution<'a>, b: &Substitution<'a>) -> Result<Substitution<'a>, String> {
        let mut new_keys = Vec::with_capacity(a.keys.len() + b.keys.len());
        new_keys.extend(a.keys.clone());
        let mut new_values = Vec::with_capacity(a.values.len() + b.values.len());
        new_values.extend(a.values.clone());

        for (i, var) in b.keys.iter().enumerate() {
            if a.keys.contains(var) {
                let a_val = a.value(var);
                let b_val = b.value(var);
                if a_val != b_val {
                    return Err(format!(
                        "Substitutions both contain variable {}, but assign different values. {} != {}",
                        var, a_val, b_val
                    ));
                }
            } else {
                new_keys.push(var);
                new_values.push(b.values[i]);
            }
        }
        Ok(Substitution {
            keys: new_keys,
            values: new_values,
            universe_constant: a.universe_constant,
        })
    }

    fn value(&self, var: &str) -> &str {
        if var == UNIVERSE_CONSTANT {
            self.universe_constant
        } else {
            let mut var_idx = 0;
            for key in &self.keys {
                if var == *key {
                    break;
                }
                var_idx += 1;
            }
            self.values[var_idx]
        }
    }

    fn insert(&mut self, var: &'a str, value: &'a str) {
        for (i, &key) in self.keys.iter().enumerate() {
            if key == var {
                if self.values.len() <= i {
                    self.values.push(value);
                } else {
                    self.values[i] = value;
                }
                return;
            }
        }
        // if key not found, insert
        self.keys.push(var);
        self.values.push(value);
    }

    fn apply(&self, terms: &[EquationContent<'a>]) -> Vec<&'a str> {
        let mut new_terms = Vec::with_capacity(terms.len());
        for term in terms {
            match term {
                EquationContent::Variable(v) => {
                    let val = if *v == UNIVERSE_CONSTANT {
                        self.universe_constant
                    } else {
                        let mut var_idx = 0;
                        for key in &self.keys {
                            if *v == *key {
                                break;
                            }
                            var_idx += 1;
                        }
                        self.values[var_idx]
                    };
                    new_terms.push(val);
                }
                EquationContent::Constant(c) => {
                    new_terms.push(*c);
                }
            }
        }
        new_terms
    }
}
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
            let mut sub = Substitution::from_vars(&var_names, w);
            for i in 0..sub_vals.len() {
                sub.values.push(sub_vals[i]);
            }
            Some(sub)
        } else {
            None
        }
    })
}
