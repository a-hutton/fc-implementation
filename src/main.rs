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

#[cfg(test)]
mod test {
    use crate::equation_parser::parse_word_equation;
    use crate::{find_solutions, print_solution};

    #[derive(Copy, Clone)]
    struct SolutionTestCase {
        equation: &'static str,
        universe: &'static str,
        should_fail: bool,
        num_solutions: usize,
    }

    #[test]
    fn test_solutions() {
        let cases = [
            // Finding correct number of solutions
            SolutionTestCase {
                equation: r#"x = "a" y z"#,
                universe: "aaab",
                should_fail: false,
                num_solutions: 15,
            },
            SolutionTestCase {
                equation: r#"(x = "a" y z && (¬y="" && ¬z=""))"#,
                universe: "aaab",
                should_fail: false,
                num_solutions: 4,
            },
            SolutionTestCase {
                equation: r#"U="aaa""#,
                universe: "aaa",
                should_fail: false,
                num_solutions: 1,
            },
            SolutionTestCase {
                equation: r#"(U=x y && (x="a" && y=x))"#,
                universe: "aa",
                should_fail: false,
                num_solutions: 1,
            },
            SolutionTestCase {
                equation: r#"(U=x y && U=x)"#,
                universe: "aa",
                should_fail: false,
                num_solutions: 1,
            },
            SolutionTestCase {
                equation: r#"(U = "a" y z &&  z="b")"#,
                universe: "aaab",
                should_fail: false,
                num_solutions: 1,
            },
            SolutionTestCase {
                equation: r#"(x = "a" y z && (x=U && z="b"))"#,
                universe: "aaab",
                should_fail: false,
                num_solutions: 1,
            },
            // Existential Quantifier
            SolutionTestCase {
                equation: r#"∃ x(U=x x)"#,
                universe: "abbabb",
                should_fail: false,
                num_solutions: 1,
            },
            SolutionTestCase {
                equation: r#"∃ x(U=x x)"#,
                universe: "abbcabb",
                should_fail: false,
                num_solutions: 0,
            },
            SolutionTestCase {
                equation: r#"(X="a" || X="aa")"#,
                universe: "aa",
                should_fail: false,
                num_solutions: 2,
            },
            // remove duplicate factors from universe
            SolutionTestCase {
                equation: r#"x="aa""#,
                universe: "aaaaaaa",
                should_fail: false,
                num_solutions: 1,
            },
            SolutionTestCase {
                equation: r#"U=U"#,
                universe: "aaaaa",
                should_fail: true,
                num_solutions: 1,
            },
            // variables with names in alphabet
            SolutionTestCase {
                equation: r#"(a="b"b && b="a")"#,
                universe: "ba",
                should_fail: false,
                num_solutions: 1,
            },
            // Syntax Errors
            SolutionTestCase {
                equation: r#"(x = "a" y z && (x=U && z="b")"#,
                universe: "aaab",
                should_fail: true,
                num_solutions: 0,
            },
            SolutionTestCase {
                equation: r#"x = "ayz"#,
                universe: "aaab",
                should_fail: true,
                num_solutions: 0,
            },
            SolutionTestCase {
                equation: r#"a y z"#,
                universe: "aaab",
                should_fail: true,
                num_solutions: 0,
            },
            SolutionTestCase {
                equation: r#"x y = a b "#,
                universe: "aaab",
                should_fail: true,
                num_solutions: 0,
            },
            // No matches
            SolutionTestCase {
                equation: r#"¬U=U"#,
                universe: "aaaaa",
                should_fail: false,
                num_solutions: 0,
            },
            SolutionTestCase {
                equation: r#"x=x"a""#,
                universe: "aaaaa",
                should_fail: false,
                num_solutions: 0,
            },
            SolutionTestCase {
                equation: r#"(x=y"aaa" && y="b")"#,
                universe: "aaaaa",
                should_fail: false,
                num_solutions: 0,
            },
        ];
        for test in cases {
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

    /// Test cases here are defined in the literature, to better fit 'real-world' use-cases
    #[test]
    fn tests_from_literature() {
        let cases = [
            // Theory of Concatenation over Finite Models - Section 1
            // 'Return all factors that occur [at least] twice in w'
            SolutionTestCase {
                equation: r#"exists p1(exists p2(exists s1(exists s2(((U=p1 x s1 && U=p2 x s2)&&¬p1=p2)))))"#,
                universe: "aabaab",
                should_fail: false,
                num_solutions: 6, // ε, a, b, aa, ab, aab
            },
            // Theory of Concatenation over Finite Models - Section 1
            // 'Return all factors x that have two non-overlapping occurrences in w'
            SolutionTestCase {
                equation: r#"exists y(exists z(y=x z x))"#,
                universe: "aabcbc",
                should_fail: false,
                num_solutions: 5, // ε, a, b, c, bc
            },
            // Theory of Concatenation over Finite Models - Example 3.5
            // 'σ(y) must occur in w between papaya and banana' (1)
            SolutionTestCase {
                equation: r#"∃ x(x = "papaya" y "banana")"#,
                universe: "papayayybanana",
                should_fail: false,
                num_solutions: 1, // yy
            },
            // Theory of Concatenation over Finite Models - Example 3.5
            // 'σ(y) must occur in w between papaya and banana' (2)
            SolutionTestCase {
                equation: r#"∃ x(x = "papaya" y "banana")"#,
                universe: "papayabannnnnnana",
                should_fail: false,
                num_solutions: 0,
            },
            // Theory of Concatenation over Finite Models - Example 3.5
            // 'w must contain papaya or banana as a factor' (1)
            SolutionTestCase {
                equation: r#"∃ x((x = "papaya"  ∨ x = "banana"))"#,
                universe: "papayabbbb",
                should_fail: false,
                num_solutions: 1, // papaya
            },
            // Theory of Concatenation over Finite Models - Example 3.5
            // 'w must contain papaya or banana as a factor' (2)
            SolutionTestCase {
                equation: r#"∃ x((x = "papaya"  ∨ x = "banana"))"#,
                universe: "bannnnnana",
                should_fail: false,
                num_solutions: 0,
            },
        ];

        for test in cases {
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

    struct FreeVarTestCase {
        equation: &'static str,
        free_vars: Vec<&'static str>,
    }

    #[test]
    fn test_free_vars() {
        let cases = [
            FreeVarTestCase {
                equation: "x = a b c",
                free_vars: vec!["a", "b", "c", "x"],
            },
            FreeVarTestCase {
                equation: "exists x(y=x)",
                free_vars: vec!["y"],
            },
        ];

        for test in cases {
            let eq = parse_word_equation(test.equation).unwrap();
            let mut got_vars = eq.free_vars();
            got_vars.sort();
            assert_eq!(got_vars, test.free_vars)
        }
    }
}
