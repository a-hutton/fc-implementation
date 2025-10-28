#![cfg(test)]
use crate::equation_parser::parse_word_equation;
use crate::{find_solutions, print_solutions};

#[derive(Copy, Clone)]
struct SolutionTestCase {
    equation: &'static str,
    universe: &'static str,
    should_fail: bool,
    num_solutions: usize,
}

/// Tests the whole process of the program - parsing and finding solutions to word equations.
/// Asserts that the predicted number of solutions matches the actual one on equations in a given
/// universe
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
        SolutionTestCase {
            equation: r#"¬((x="" || x="ab") || (x="a" || x="b"))"#,
            universe: "ab",
            should_fail: false,
            num_solutions: 0,
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
        SolutionTestCase {
            equation: r#"forall x (x="a")"#,
            universe: "aa",
            should_fail: false,
            num_solutions: 0,
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
                print_solutions(&solutions, test.universe);
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
        // Theory of Concatenation over Finite Models - Example 3.5
        // 'σ(x) occurs exactly once in w'
        SolutionTestCase {
            equation: r#"(x="bbb" && ∃ p(∃ s((U=p x s ∧ ¬∃ ph(∃ sh((U=ph x sh ∧ ¬ph=p)))))))"#,
            universe: "ababbba",
            should_fail: false,
            num_solutions: 1,
        },
        SolutionTestCase {
            equation: r#"(x="bbb" && ∃ p(∃ s((U=p x s ∧ ¬∃ ph(∃ sh((U=ph x sh ∧ ¬ph=p)))))))"#,
            universe: "abb",
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
                print_solutions(&solutions, test.universe);
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
