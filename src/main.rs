mod equation_parser;
mod equations;

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
        print!("{}: {:width$}", key, sub[**key], width = universe_len);
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

#[cfg(test)]
mod test {
    use crate::equation_parser::parse_word_equation;
    use crate::{find_solutions, print_solution};

    #[derive(Copy, Clone)]
    struct TestRes {
        equation: &'static str,
        universe: &'static str,
        should_fail: bool,
        num_solutions: usize,
    }

    static CASES: [TestRes; 16] = [
        // Finding correct number of solutions
        TestRes {
            equation: r#"x = "a" y z"#,
            universe: "aaab",
            should_fail: false,
            num_solutions: 15,
        },
        TestRes {
            equation: r#"(x = "a" y z && (¬y="" && ¬z=""))"#,
            universe: "aaab",
            should_fail: false,
            num_solutions: 4,
        },
        TestRes {
            equation: r#"U="aaa""#,
            universe: "aaa",
            should_fail: false,
            num_solutions: 1,
        },
        TestRes {
            equation: r#"(U=x y && (x="a" && y=x))"#,
            universe: "aa",
            should_fail: false,
            num_solutions: 1,
        },
        TestRes {
            equation: r#"(U=x y && U=x)"#,
            universe: "aa",
            should_fail: false,
            num_solutions: 1,
        },
        TestRes {
            equation: r#"(U = "a" y z &&  z="b")"#,
            universe: "aaab",
            should_fail: false,
            num_solutions: 1,
        },
        TestRes {
            equation: r#"(x = "a" y z && (x=U && z="b"))"#,
            universe: "aaab",
            should_fail: false,
            num_solutions: 1,
        },
        // remove duplicate factors from universe
        TestRes {
            equation: r#"x="aa""#,
            universe: "aaaaaaa",
            should_fail: false,
            num_solutions: 1,
        },
        // variables with names in alphabet
        TestRes {
            equation: r#"(a="b"b && b="a")"#,
            universe: "ba",
            should_fail: false,
            num_solutions: 1,
        },
        // Syntax Errors
        TestRes {
            equation: r#"(x = "a" y z && (x=U && z="b")"#,
            universe: "aaab",
            should_fail: true,
            num_solutions: 0,
        },
        TestRes {
            equation: r#"x = "ayz"#,
            universe: "aaab",
            should_fail: true,
            num_solutions: 0,
        },
        TestRes {
            equation: r#"a y z"#,
            universe: "aaab",
            should_fail: true,
            num_solutions: 0,
        },
        TestRes {
            equation: r#"x y = a b "#,
            universe: "aaab",
            should_fail: true,
            num_solutions: 0,
        },
        // No matches
        TestRes {
            equation: r#"¬U=U"#,
            universe: "aaaaa",
            should_fail: false,
            num_solutions: 0,
        },
        TestRes {
            equation: r#"x=x"a""#,
            universe: "aaaaa",
            should_fail: false,
            num_solutions: 0,
        },
        TestRes {
            equation: r#"(x=y"aaa" && y="b")"#,
            universe: "aaaaa",
            should_fail: false,
            num_solutions: 0,
        },
    ];

    #[test]
    fn test_solutions() {
        for test in CASES {
            println!("Finding solutions for '{}'", test.equation);
            let res = parse_word_equation(test.equation);
            match res {
                None => assert!(test.should_fail),
                Some(equation) => {
                    let solutions = find_solutions(&*equation, test.universe);
                    println!("Found {n} solutions", n = solutions.len());
                    for sol in solutions.iter() {
                        print_solution(sol);
                    }
                    assert_eq!(solutions.len(), test.num_solutions);
                }
            }
            println!("-------- Test Passed --------\n")
        }
    }
}
