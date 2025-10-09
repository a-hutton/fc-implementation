use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::hash::Hash;

fn main() {
    let phi = AtomicWordEquation::new(
        "phi",
        vec![
            AtomicContent::FreeVariable("x"),
            AtomicContent::Constant("aaa"),
            AtomicContent::FreeVariable("y"),
        ],
    );
    let rho = AtomicWordEquation::new("x", vec![AtomicContent::Constant("a")]);

    let conj_form = ConjunctionWordEquation::new(Box::from(phi), Box::from(rho));

    let sub = HashMap::from([
        ("U", "aaaaaaaabbbb"),
        ("x", "a"),
        ("y", "b"),
        ("phi", "aaaab"),
    ]);

    println!("Formula: {}", conj_form);
    println!("Free vars: {:?}", conj_form.free_vars());
    println!("Substitution: {:?}", sub);
    println!(
        "Substitution holds: {:?}",
        conj_form.check_substitution(&sub)
    );
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

trait WordEquation: fmt::Display {
    fn free_vars(&self) -> Vec<&'static str>;
    fn check_substitution(&self, substitution: &Substitution) -> bool;
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
}

impl WordEquation for AtomicWordEquation {
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
            panic!("Missing universe variable `U` (𝔲) in substitution")
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
        write!(f, "{}≐", self.lhs_variable).expect("TODO: panic message");
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

/// φ∧ψ
struct ConjunctionWordEquation {
    lhs: Box<dyn WordEquation>,
    rhs: Box<dyn WordEquation>,
}
impl ConjunctionWordEquation {
    fn new(lhs: Box<dyn WordEquation>, rhs: Box<dyn WordEquation>) -> ConjunctionWordEquation {
        ConjunctionWordEquation { lhs, rhs }
    }
}

impl WordEquation for ConjunctionWordEquation {
    fn free_vars(&self) -> Vec<&'static str> {
        let mut free = vec![];
        free.extend(self.lhs.free_vars());
        free.extend(self.rhs.free_vars());

        free
    }

    fn check_substitution(&self, substitution: &Substitution) -> bool {
        // check substitution holds for lhs and rhs. Contradictions?
        self.lhs.check_substitution(substitution) && self.rhs.check_substitution(substitution)
    }
}

impl fmt::Display for ConjunctionWordEquation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({} ∧ {})", self.lhs, self.rhs).expect("TODO: panic message");

        Ok(())
    }
}
