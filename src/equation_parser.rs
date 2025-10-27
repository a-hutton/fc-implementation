use crate::equations::{
    AtomicContent, AtomicWordEquation, ConjunctionWordEquation, DisjunctionWordEquation,
    ExistentialEquation, NegatedEquation, UniversalEquation, WordEquation,
};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "word_equation.pest"]
struct WordEquationParser;

#[test]
fn test_parser() {
    let test_cases = [
        r#"x = "a" yqqq z"#,
        r#"¬x = "a" yqqq z"#,
        r#"(x=y"abc" && ¬y="m")"#,
    ];
    for case in test_cases {
        let rules = WordEquationParser::parse(Rule::word_equation, case)
            .expect("should parse")
            .next()
            .unwrap();
        println!("Success {}", case);
        println!("{:#?}", rules);
    }
}

pub fn parse_word_equation(eq: &str) -> Option<Box<dyn WordEquation + '_>> {
    let res = WordEquationParser::parse(Rule::word_equation, eq);
    if let Ok(mut equation) = res {
        let equation = equation.next().unwrap();
        let equation = equation.into_inner().next().unwrap();
        let equation = parse_equation_pair(equation);
        Some(equation)
    } else {
        eprintln!("Parse error on word equation\n{:?}", res);
        None
    }
}

#[cfg(test)]
#[test]
fn test_equation_parse_creator() {
    let test_cases = [
        r#"x = "a" yqqq z"#,
        r#"¬q = "a""#,
        r#"¬x = "a" yqqq z"#,
        r#"(x=y"abc" && ¬y="")"#,
        r#"exists x (U=x x)"#,
        r#"¬(x="aa"||x="bb")"#,
    ];
    for case in test_cases {
        let eq = parse_word_equation(case);
        match eq {
            None => {
                panic!("Parse error on word equation\n{:?}", eq)
            }
            Some(val) => {
                println!("Success {}\n---------------", val);
            }
        }
    }
}

fn parse_atomic_equation<'a>(eq: &mut Pairs<'a, Rule>) -> AtomicWordEquation<'a> {
    let lhs_var = eq.next().unwrap().as_str();
    let rhs = eq
        .map(|x| match x.as_rule() {
            Rule::constant => {
                let str = x.as_str();
                // remove " at start and end
                AtomicContent::Constant(str[1..str.len() - 1].into())
            }
            Rule::variable => AtomicContent::FreeVariable(x.as_str()),
            _ => panic!("Unreachable state"),
        })
        .collect();

    AtomicWordEquation::new(lhs_var, rhs)
}

fn parse_negated_equation<'a>(eq: &mut Pairs<'a, Rule>) -> NegatedEquation<'a> {
    let inner = eq.next().unwrap().into_inner().next().unwrap();
    let atomic_eq = parse_equation_pair(inner);
    NegatedEquation::new(atomic_eq)
}

fn parse_conjunctive_equation<'a>(eq: &mut Pairs<'a, Rule>) -> ConjunctionWordEquation<'a> {
    let lhs_eq = eq.next().unwrap();
    if lhs_eq.as_rule() != Rule::word_equation {
        panic!(
            "Parse error on conjunctive equation. Expected a word equation, got {:?}\n{:?}",
            lhs_eq.as_rule(),
            lhs_eq
        )
    }
    let lhs_eq = lhs_eq.into_inner().next().unwrap();

    let rhs_eq = eq.next().unwrap();
    if rhs_eq.as_rule() != Rule::word_equation {
        panic!(
            "Parse error on conjunctive equation. Expected a word equation, got {:?}\n{:?}",
            rhs_eq.as_rule(),
            rhs_eq
        )
    }
    let rhs_eq = rhs_eq.into_inner().next().unwrap();

    let lhs = parse_equation_pair(lhs_eq);
    let rhs = parse_equation_pair(rhs_eq);

    ConjunctionWordEquation::new(lhs, rhs)
}

fn parse_disjunctive_equation<'a>(eq: &mut Pairs<'a, Rule>) -> DisjunctionWordEquation<'a> {
    let lhs_eq = eq.next().unwrap();
    if lhs_eq.as_rule() != Rule::word_equation {
        panic!(
            "Parse error on disjunctive equation. Expected a word equation, got {:?}\n{:?}",
            lhs_eq.as_rule(),
            lhs_eq
        )
    }
    let lhs_eq = lhs_eq.into_inner().next().unwrap();

    let rhs_eq = eq.next().unwrap();
    if rhs_eq.as_rule() != Rule::word_equation {
        panic!(
            "Parse error on disjunctive equation. Expected a word equation, got {:?}\n{:?}",
            rhs_eq.as_rule(),
            rhs_eq
        )
    }
    let rhs_eq = rhs_eq.into_inner().next().unwrap();

    let lhs = parse_equation_pair(lhs_eq);
    let rhs = parse_equation_pair(rhs_eq);

    DisjunctionWordEquation::new(lhs, rhs)
}

fn parse_existential_equation<'a>(eq: &mut Pairs<'a, Rule>) -> ExistentialEquation<'a> {
    let variable_pair = eq.next().unwrap();
    if variable_pair.as_rule() != Rule::variable {
        panic!("Expected a variable to be bound in 'exists' clause")
    }
    let bound_variable = variable_pair.as_str();

    // 'inner' is the wrapping word_equation rule, so we go into that to get the equation type
    let inner = eq.next().unwrap().into_inner().next().unwrap();
    let inner_equation = parse_equation_pair(inner);

    ExistentialEquation::new(bound_variable, inner_equation)
}

fn parse_universal_equation<'a>(eq: &mut Pairs<'a, Rule>) -> UniversalEquation<'a> {
    let variable_pair = eq.next().unwrap();
    if variable_pair.as_rule() != Rule::variable {
        panic!("Expected a variable to be bound in 'exists' clause")
    }
    let bound_variable = variable_pair.as_str();

    // 'inner' is the wrapping word_equation rule, so we go into that to get the equation type
    let inner = eq.next().unwrap().into_inner().next().unwrap();
    let inner_equation = parse_equation_pair(inner);

    UniversalEquation::new(bound_variable, inner_equation)
}

/// Parse a pest `Pair` into a [`WordEquation`] `Box` pointer with the appropriate
/// implementation of [`WordEquation`]
fn parse_equation_pair<'a>(pair: Pair<'a, Rule>) -> Box<dyn WordEquation + 'a> {
    match pair.as_rule() {
        Rule::neg_atomic => Box::from(parse_negated_equation(&mut pair.into_inner())),
        Rule::atomic => Box::from(parse_atomic_equation(&mut pair.into_inner())),
        Rule::conjunctive_equation => Box::from(parse_conjunctive_equation(&mut pair.into_inner())),
        Rule::disjunctive_equation => Box::from(parse_disjunctive_equation(&mut pair.into_inner())),
        Rule::existential_equation => Box::from(parse_existential_equation(&mut pair.into_inner())),
        Rule::universal_equation => Box::from(parse_universal_equation(&mut pair.into_inner())),

        r => {
            panic!("Expected an equation type, round rule type {:?}", r);
        }
    }
}
