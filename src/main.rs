use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::hash::Hash;

fn main() {
    let x = AtomicWordEquation::new(
        "phi",
        vec![
            AtomicContent::FreeVariable("x"),
            AtomicContent::Constant("aaa"),
            AtomicContent::FreeVariable("y"),
        ],
    );

    let sub = HashMap::from([
        ("U", "aaaaaaaabbbb"),
        ("x", ""),
        ("y", "b"),
        ("phi", "aaab"),
    ]);

    println!("Formula: {}", x);
    println!("Free vars: {:?}", x.free_vars());
    println!("Substitution: {:?}", sub);
    println!("Substitution holds: {:?}", x.check_substitution(&sub));
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
            factors.push(substr);
        }
    }
    factors.push(w);
    factors
}

type Substitution = HashMap<&'static str, &'static str>;

#[derive(Debug, Hash)]
enum AtomicContent {
    FreeVariable(&'static str),
    Constant(&'static str),
}

fn substitute(equation: &Vec<AtomicContent>, substitution: &Substitution) -> Vec<&'static str> {
    let mut new_terms = Vec::with_capacity(equation.len());
    for term in equation {
        match term {
            AtomicContent::FreeVariable(v) => {
                if !substitution.contains_key(v) {
                    panic!("No substitution for variable {:?}", v);
                }
                let val = substitution[*v];
                new_terms.push(val);
            }
            AtomicContent::Constant(c) => {
                new_terms.push(*c);
            }
        }
    }
    new_terms
}

/// x = abc
/// abc is a vector of `AtomicContent` -- a sequence of either variables whose values
/// can be provided by substitutions, or constants in the universe
#[derive(Debug)]
struct AtomicWordEquation {
    lhs_variable: &'static str,
    rhs: Vec<AtomicContent>,
}

impl AtomicWordEquation {
    fn new(lhs: &'static str, rhs: Vec<AtomicContent>) -> AtomicWordEquation {
        AtomicWordEquation {
            lhs_variable: lhs,
            rhs,
        }
    }

    fn free_vars(&self) -> Vec<&'static str> {
        let mut free = vec![];
        if self.lhs_variable != "U" {
            free.push(self.lhs_variable);
        }

        for content in &self.rhs {
            if let AtomicContent::FreeVariable(var) = content {
                free.push(var);
            }
        }
        free
    }

    fn check_substitution(&self, substitution: &Substitution) -> bool {
        if !substitution.contains_key("U") {
            panic!("Missing universe variable `U` in substitution")
        }
        println!("σ(U)={:?}", substitution["U"]);
        let universe = generate_factors(substitution["U"]);
        for (key, val) in substitution.iter() {
            if !universe.contains(val) {
                panic!(
                    "Substitution {} for variable {} is not in the universe",
                    val, key
                );
            }
        }

        let lhs_vec = vec![AtomicContent::FreeVariable(self.lhs_variable)];
        let lhs_sub = substitute(&lhs_vec, substitution).join("");
        let rhs_sub = substitute(&self.rhs, substitution).join("");

        lhs_sub == rhs_sub
    }
}

impl fmt::Display for AtomicWordEquation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}=", self.lhs_variable).expect("TODO: panic message");
        for content in &self.rhs {
            match content {
                AtomicContent::FreeVariable(v) => {
                    write!(f, "{}", v).expect("TODO: panic message");
                }
                AtomicContent::Constant(c) => {
                    write!(f, "{:?}", c).expect("TODO: panic message");
                }
            }
        }
        Ok(())
    }
}
