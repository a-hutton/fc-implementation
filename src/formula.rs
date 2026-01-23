use crate::Substitution;
use std::fmt;
use std::fmt::Formatter;

pub const UNIVERSE_CONSTANT: &str = "$U";

/// Represents values that can appear in an 'atomic' word equation - either a variable with a name,
/// or a string constant
#[derive(Debug, Hash)]
pub enum EquationContent<'a> {
    Variable(&'a str),
    Constant(&'a str),
}

/// Applies a [`Substitution`] to a `Vec` of [`EquationContent`] - either variables or constants.
/// Constants do not have their values changed, variables are given their respective values from
/// the provided [`Substitution`]
fn substitute<'a>(
    terms: &Vec<EquationContent<'a>>,
    substitution: &Substitution<'a>,
) -> Vec<&'a str> {
    let mut new_terms = Vec::with_capacity(terms.len());
    for term in terms {
        match term {
            EquationContent::Variable(v) => {
                if !substitution.contains_key(v) {
                    panic!("No substitution for (free) variable {:?}", v);
                }
                let val = substitution[*v];
                new_terms.push(val);
            }
            EquationContent::Constant(c) => {
                new_terms.push(*c);
            }
        }
    }
    new_terms
}

/// A common interface for all word formula types
pub trait Formula: fmt::Display + fmt::Debug {
    fn free_vars(&self) -> Vec<&str>;
    fn check_substitution(&self, substitution: &Substitution, universe: &[&str]) -> bool;
}

/// An 'atomic' equation where the left hand side is a single variable, and the right hand side is
/// a sequence of [`EquationContent`]: either variables or constants
#[derive(Debug)]
pub struct AtomicWordEquation<'a> {
    lhs_variable: &'a str,
    rhs: Vec<EquationContent<'a>>,
}

impl AtomicWordEquation<'_> {
    pub fn new<'a>(lhs: &'a str, rhs: Vec<EquationContent<'a>>) -> AtomicWordEquation<'a> {
        AtomicWordEquation {
            lhs_variable: lhs,
            rhs,
        }
    }
}

impl<'a> Formula for AtomicWordEquation<'a> {
    fn free_vars(&self) -> Vec<&'a str> {
        let mut free = vec![];
        if self.lhs_variable != UNIVERSE_CONSTANT {
            free.push(self.lhs_variable);
        }

        for content in &self.rhs {
            // if term is a free variable, add to vec if not already there
            if let EquationContent::Variable(var) = content
                && !free.contains(var)
                && *var != UNIVERSE_CONSTANT
            {
                free.push(*var);
            }
        }
        free
    }

    /// The simple atomic case, where the left and right -hand sides are replaced using
    /// [`substitute`] and compared with simple string comparison
    fn check_substitution(&self, substitution: &Substitution, universe: &[&str]) -> bool {
        if !substitution.contains_key(UNIVERSE_CONSTANT) {
            panic!("Missing universe constant `$U` (𝔲) in substitution")
        }
        for (key, val) in substitution.iter() {
            if !universe.contains(val) {
                panic!(
                    "Substitution {} for variable {} is not in the universe",
                    val, key
                );
            }
        }

        let lhs_vec = vec![EquationContent::Variable(self.lhs_variable)];
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
                EquationContent::Variable(v) => {
                    write!(f, "{} ", v).expect("TODO: panic message");
                }
                EquationContent::Constant(c) => {
                    write!(f, "{:?} ", c).expect("TODO: panic message");
                }
            }
        }
        Ok(())
    }
}

/// A formula φ∧ψ: the conjunction of two sub-formulas
#[derive(Debug)]
pub struct ConjunctiveFormula<'a> {
    lhs: Box<dyn Formula + 'a>,
    rhs: Box<dyn Formula + 'a>,
}
impl ConjunctiveFormula<'_> {
    pub fn new<'a>(
        lhs: Box<dyn Formula + 'a>,
        rhs: Box<dyn Formula + 'a>,
    ) -> ConjunctiveFormula<'a> {
        ConjunctiveFormula { lhs, rhs }
    }
}

impl Formula for ConjunctiveFormula<'_> {
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

    /// Checks if the substitution holds for _both_ the left and right -hand components of
    /// the disjunction
    fn check_substitution(&self, substitution: &Substitution, universe: &[&str]) -> bool {
        // check substitution holds for lhs and rhs. Contradictions?
        let lhs_holds = self.lhs.check_substitution(substitution, universe);
        let rhs_holds = self.rhs.check_substitution(substitution, universe);
        lhs_holds && rhs_holds
    }
}

impl fmt::Display for ConjunctiveFormula<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({} ∧ {})", self.lhs, self.rhs)
    }
}

/// A formula φ∨ψ: the disjunction of two sub-formulas
#[derive(Debug)]
pub struct DisjunctiveFormula<'a> {
    lhs: Box<dyn Formula + 'a>,
    rhs: Box<dyn Formula + 'a>,
}
impl DisjunctiveFormula<'_> {
    pub fn new<'a>(
        lhs: Box<dyn Formula + 'a>,
        rhs: Box<dyn Formula + 'a>,
    ) -> DisjunctiveFormula<'a> {
        DisjunctiveFormula { lhs, rhs }
    }
}

impl Formula for DisjunctiveFormula<'_> {
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

    /// Checks if the substitution holds for _either_ the left or right -hand components of
    /// the conjunction
    fn check_substitution(&self, substitution: &Substitution, universe: &[&str]) -> bool {
        // check substitution holds for lhs and rhs. Contradictions?
        let lhs_holds = self.lhs.check_substitution(substitution, universe);
        let rhs_holds = self.rhs.check_substitution(substitution, universe);
        lhs_holds || rhs_holds
    }
}

impl fmt::Display for DisjunctiveFormula<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({} ∨ {})", self.lhs, self.rhs)
    }
}

/// A formula ¬φ: the negation of a single sub-formulas
#[derive(Debug)]
pub struct NegativeFormula<'a> {
    inner: Box<dyn Formula + 'a>,
}

impl NegativeFormula<'_> {
    pub fn new<'a>(inner: Box<dyn Formula + 'a>) -> NegativeFormula {
        NegativeFormula { inner }
    }
}

impl Formula for NegativeFormula<'_> {
    fn free_vars(&self) -> Vec<&str> {
        self.inner.free_vars()
    }

    /// Simply the negation of the [`Formula::check_substitution`] of the sub-formula
    fn check_substitution(&self, substitution: &Substitution, universe: &[&str]) -> bool {
        !self.inner.check_substitution(substitution, universe)
    }
}

impl fmt::Display for NegativeFormula<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "¬").expect("TODO: panic message");
        self.inner.fmt(f)
    }
}

/// A formula ∃ x: φ(x): where a single variable is bound by an existential quantifier
#[derive(Debug)]
pub struct ExistentialFormula<'a> {
    inner: Box<dyn Formula + 'a>,
    bound_var: &'a str,
}

impl<'a> ExistentialFormula<'a> {
    pub fn new(bound_var: &'a str, inner_formula: Box<dyn Formula + 'a>) -> Self {
        ExistentialFormula {
            inner: inner_formula,
            bound_var,
        }
    }
}

impl Formula for ExistentialFormula<'_> {
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

    /// Checks a [`Substitution`] by iterating over the given universe, and for each possible value
    /// to assign to the bound variable _x_, a new substitution is checked on the inner
    /// [`Formula::check_substitution`], returning `true` when the first valid substitution is found
    fn check_substitution(&self, substitution: &Substitution, universe: &[&str]) -> bool {
        let mut altered_substitution = substitution.clone();
        for factor in universe {
            altered_substitution.insert(self.bound_var, factor);
            let holds = self
                .inner
                .check_substitution(&altered_substitution, universe);
            if holds {
                return true;
            }
        }
        false
    }
}

impl fmt::Display for ExistentialFormula<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let _ = write!(f, "∃{x}: (", x = self.bound_var);
        let _ = self.inner.fmt(f);
        write!(f, ")")
    }
}

/// A formula ∀ x: φ(x): where a single variable is bound by a universal quantifier
#[derive(Debug)]
pub struct UniversalFormula<'a> {
    inner: Box<dyn Formula + 'a>,
    bound_var: &'a str,
}

impl<'a> UniversalFormula<'a> {
    pub fn new(bound_var: &'a str, inner_formula: Box<dyn Formula + 'a>) -> Self {
        UniversalFormula {
            inner: inner_formula,
            bound_var,
        }
    }
}

impl Formula for UniversalFormula<'_> {
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

    /// Checks a [`Substitution`] by iterating over the given universe, and for each possible value
    /// to assign to the bound variable _x_, a new substitution is checked on the inner
    /// [`Formula::check_substitution`], returning `true` if every new substitution holds
    fn check_substitution(&self, substitution: &Substitution, universe: &[&str]) -> bool {
        let mut altered_substitution = substitution.clone();
        for factor in universe {
            altered_substitution.insert(self.bound_var, factor);
            let holds = self
                .inner
                .check_substitution(&altered_substitution, universe);
            if holds {
                return false;
            }
        }
        true
    }
}

impl fmt::Display for UniversalFormula<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let _ = write!(f, "∀{x}: (", x = self.bound_var);
        let _ = self.inner.fmt(f);
        write!(f, ")")
    }
}
