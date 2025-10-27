use crate::{generate_factors, Substitution};
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Hash)]
pub enum AtomicContent<'a> {
    FreeVariable(&'a str),
    Constant(&'a str),
}

fn substitute<'a>(
    equation: &Vec<AtomicContent<'a>>,
    substitution: &Substitution<'a>,
) -> Vec<&'a str> {
    let mut new_terms = Vec::with_capacity(equation.len());
    for term in equation {
        match term {
            AtomicContent::FreeVariable(v) => {
                if !substitution.contains_key(v) {
                    panic!("No substitution for (free) variable {:?}", v);
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

pub trait WordEquation: fmt::Display + fmt::Debug {
    fn free_vars(&self) -> Vec<&str>;
    fn check_substitution(&self, substitution: &Substitution) -> bool;
}

/// x = abc
/// abc is a vector of [`AtomicContent`] -- a sequence of either variables whose values
/// can be provided by substitutions, or constants in the universe
#[derive(Debug)]
pub struct AtomicWordEquation<'a> {
    lhs_variable: &'a str,
    rhs: Vec<AtomicContent<'a>>,
}

impl AtomicWordEquation<'_> {
    pub fn new<'a>(lhs: &'a str, rhs: Vec<AtomicContent<'a>>) -> AtomicWordEquation<'a> {
        AtomicWordEquation {
            lhs_variable: lhs,
            rhs,
        }
    }
}

impl<'a> WordEquation for AtomicWordEquation<'a> {
    fn free_vars(&self) -> Vec<&'a str> {
        let mut free = vec![];
        if self.lhs_variable != "U" {
            free.push(self.lhs_variable);
        }

        for content in &self.rhs {
            // if term is a free variable, add to vec if not already there
            if let AtomicContent::FreeVariable(var) = content
                && !free.contains(var)
                && *var != "U"
            {
                free.push(*var);
            }
        }
        free
    }

    fn check_substitution(&self, substitution: &Substitution) -> bool {
        if !substitution.contains_key("U") {
            panic!("Missing universe variable `U` (𝔲) in substitution")
        }
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

impl fmt::Display for AtomicWordEquation<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}≐", self.lhs_variable).expect("TODO: panic message");
        for content in &self.rhs {
            match content {
                AtomicContent::FreeVariable(v) => {
                    write!(f, "{} ", v).expect("TODO: panic message");
                }
                AtomicContent::Constant(c) => {
                    write!(f, "{:?} ", c).expect("TODO: panic message");
                }
            }
        }
        Ok(())
    }
}

/// φ∧ψ
#[derive(Debug)]
pub struct ConjunctionWordEquation<'a> {
    lhs: Box<dyn WordEquation + 'a>,
    rhs: Box<dyn WordEquation + 'a>,
}
impl ConjunctionWordEquation<'_> {
    pub fn new<'a>(
        lhs: Box<dyn WordEquation + 'a>,
        rhs: Box<dyn WordEquation + 'a>,
    ) -> ConjunctionWordEquation<'a> {
        ConjunctionWordEquation { lhs, rhs }
    }
}

impl WordEquation for ConjunctionWordEquation<'_> {
    fn free_vars(&self) -> Vec<&str> {
        let mut free = vec![];
        free.extend(self.lhs.free_vars());
        for var in self.rhs.free_vars() {
            if !free.contains(&var) {
                free.push(var);
            }
        }
        free
    }

    fn check_substitution(&self, substitution: &Substitution) -> bool {
        // check substitution holds for lhs and rhs. Contradictions?
        let lhs_holds = self.lhs.check_substitution(substitution);
        let rhs_holds = self.rhs.check_substitution(substitution);
        lhs_holds && rhs_holds
    }
}

impl fmt::Display for ConjunctionWordEquation<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({} ∧ {})", self.lhs, self.rhs)
    }
}

/// φ∧ψ
#[derive(Debug)]
pub struct DisjunctionWordEquation<'a> {
    lhs: Box<dyn WordEquation + 'a>,
    rhs: Box<dyn WordEquation + 'a>,
}
impl DisjunctionWordEquation<'_> {
    pub fn new<'a>(
        lhs: Box<dyn WordEquation + 'a>,
        rhs: Box<dyn WordEquation + 'a>,
    ) -> DisjunctionWordEquation<'a> {
        DisjunctionWordEquation { lhs, rhs }
    }
}

impl WordEquation for DisjunctionWordEquation<'_> {
    fn free_vars(&self) -> Vec<&str> {
        let mut free = vec![];
        free.extend(self.lhs.free_vars());
        for var in self.rhs.free_vars() {
            if !free.contains(&var) {
                free.push(var);
            }
        }
        free
    }

    fn check_substitution(&self, substitution: &Substitution) -> bool {
        // check substitution holds for lhs and rhs. Contradictions?
        let lhs_holds = self.lhs.check_substitution(substitution);
        let rhs_holds = self.rhs.check_substitution(substitution);
        lhs_holds || rhs_holds
    }
}

impl fmt::Display for DisjunctionWordEquation<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({} ∨ {})", self.lhs, self.rhs)
    }
}

#[derive(Debug)]
pub struct NegatedEquation<'a> {
    inner: Box<dyn WordEquation + 'a>,
}

impl NegatedEquation<'_> {
    pub fn new<'a>(inner: Box<dyn WordEquation + 'a>) -> NegatedEquation {
        NegatedEquation { inner }
    }
}

impl WordEquation for NegatedEquation<'_> {
    fn free_vars(&self) -> Vec<&str> {
        self.inner.free_vars()
    }

    fn check_substitution(&self, substitution: &Substitution) -> bool {
        !self.inner.check_substitution(substitution)
    }
}

impl fmt::Display for NegatedEquation<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "¬").expect("TODO: panic message");
        self.inner.fmt(f)
    }
}

#[derive(Debug)]
pub struct ExistentialEquation<'a> {
    inner: Box<dyn WordEquation + 'a>,
    bound_var: &'a str,
}

impl<'a> ExistentialEquation<'a> {
    pub fn new(bound_var: &'a str, inner_equation: Box<dyn WordEquation + 'a>) -> Self {
        ExistentialEquation {
            inner: inner_equation,
            bound_var,
        }
    }
}

impl WordEquation for ExistentialEquation<'_> {
    fn free_vars(&self) -> Vec<&str> {
        let inner_free = self.inner.free_vars();
        let mut free = Vec::with_capacity(inner_free.len());
        for var in inner_free {
            if var != self.bound_var {
                free.push(var);
            }
        }
        free
    }

    fn check_substitution(&self, substitution: &Substitution) -> bool {
        let universe_word = substitution["U"];
        let all_factors = generate_factors(universe_word);
        let mut altered_substitution = substitution.clone();
        for factor in all_factors {
            altered_substitution.insert(self.bound_var, factor);
            let holds = self.inner.check_substitution(&altered_substitution);
            if holds {
                return true;
            }
        }
        false
    }
}

impl fmt::Display for ExistentialEquation<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let _ = write!(f, "∃{x}: (", x = self.bound_var);
        let _ = self.inner.fmt(f);
        write!(f, ")")
    }
}

#[derive(Debug)]
pub struct UniversalEquation<'a> {
    inner: Box<dyn WordEquation + 'a>,
    bound_var: &'a str,
}

impl<'a> UniversalEquation<'a> {
    pub fn new(bound_var: &'a str, inner_equation: Box<dyn WordEquation + 'a>) -> Self {
        UniversalEquation {
            inner: inner_equation,
            bound_var,
        }
    }
}

impl WordEquation for UniversalEquation<'_> {
    fn free_vars(&self) -> Vec<&str> {
        let inner_free = self.inner.free_vars();
        let mut free = Vec::with_capacity(inner_free.len());
        for var in inner_free {
            if var != self.bound_var {
                free.push(var);
            }
        }
        free
    }

    fn check_substitution(&self, substitution: &Substitution) -> bool {
        let universe_word = substitution["U"];
        let all_factors = generate_factors(universe_word);
        let mut altered_substitution = substitution.clone();
        for factor in all_factors {
            altered_substitution.insert(self.bound_var, factor);
            let holds = self.inner.check_substitution(&altered_substitution);
            if holds {
                return false;
            }
        }
        true
    }
}

impl fmt::Display for UniversalEquation<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let _ = write!(f, "∀{x}: (", x = self.bound_var);
        let _ = self.inner.fmt(f);
        write!(f, ")")
    }
}
