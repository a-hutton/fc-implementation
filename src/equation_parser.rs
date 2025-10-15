use crate::equation_parser;
use crate::equations::{
    AtomicContent, AtomicWordEquation, ConjunctionWordEquation, NegatedEquation, WordEquation,
};
use pest::iterators::Pairs;
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
        let rules = WordEquationParser::parse(Rule::word_equation, &case)
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
        match equation.as_rule() {
            Rule::atomic => {
                let atomic = parse_atomic_equation(&mut equation.into_inner());
                println!("{:#?}", atomic);
                Some(Box::from(atomic))
            }
            Rule::neg_atomic => {
                let neg_atomic = parse_negated_equation(&mut equation.into_inner());
                println!("{:#?}", neg_atomic);
                Some(Box::from(neg_atomic))
            }
            Rule::conjunctive_equation => {
                let conj = parse_conjunctive_equation(&mut equation.into_inner());
                println!("{:#?}", conj);
                Some(Box::from(conj))
            }
            _ => panic!("Expected an equation: {:#?}", equation),
        }
    } else {
        eprintln!("Parse error on word equation\n{:?}", res);
        None
    }
}

#[test]
fn test_equation_parse_creator() {
    let test_cases = [
        r#"x = "a" yqqq z"#,
        r#"¬q = "a""#,
        r#"¬x = "a" yqqq z"#, //
        r#"(x=y"abc" && ¬y="")"#,
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
    let mut inner = eq.next().unwrap().into_inner();
    let atomic_eq = parse_atomic_equation(&mut inner);
    NegatedEquation::new(atomic_eq)
}

fn parse_conjunctive_equation<'a>(
    eq: &mut pest::iterators::Pairs<'a, equation_parser::Rule>,
) -> ConjunctionWordEquation<'a> {
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

    let lhs: Box<dyn WordEquation> = match lhs_eq.as_rule() {
        Rule::neg_atomic => Box::from(parse_negated_equation(&mut lhs_eq.into_inner())),
        Rule::atomic => Box::from(parse_atomic_equation(&mut lhs_eq.into_inner())),
        Rule::conjunctive_equation => {
            Box::from(parse_conjunctive_equation(&mut lhs_eq.into_inner()))
        }
        r => {
            panic!("Unreachable, found rule {:?}", r);
        }
    };

    let rhs: Box<dyn WordEquation> = match rhs_eq.as_rule() {
        Rule::neg_atomic => Box::from(parse_negated_equation(&mut rhs_eq.into_inner())),
        Rule::atomic => Box::from(parse_atomic_equation(&mut rhs_eq.into_inner())),
        Rule::conjunctive_equation => {
            Box::from(parse_conjunctive_equation(&mut rhs_eq.into_inner()))
        }
        r => {
            panic!("Unreachable, found rule {:?}", r);
        }
    };

    ConjunctionWordEquation::new(lhs, rhs)
}
