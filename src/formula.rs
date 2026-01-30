use crate::Substitution;
use itertools::Itertools;
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

#[derive(PartialEq, Clone, Eq, Hash, Debug)]
pub enum VariableRelation<'a> {
    /// Constraint that for a given equation, the length of the lhs variable (`x`) must
    /// be equal to the sum of the length of the rhs variables (`sum_vars`) plus the length
    /// of any constants (`c`)
    LengthEquality {
        lhs: &'a str,
        rhs_vars: Vec<&'a str>,
        c: usize,
    },
    /// Set of constraints that must _all_ hold
    Conjunction {
        constraints: Vec<VariableRelation<'a>>,
    },
    /// Set of constraints, of which (at least) _one_ must hold
    Disjunction {
        constraints: Vec<VariableRelation<'a>>,
    },
    Negation {
        inner: Box<VariableRelation<'a>>,
    },

    /// The (value of) variable with name `prefix_var` appears at the start of `lhs`
    VarPrefix {
        lhs: &'a str,
        prefix_var: &'a str,
    },
    /// The string `prefix` appears at the start of the value of variable `lhs`
    ConstPrefix {
        lhs: &'a str,
        prefix: &'a str,
    },
}

impl<'a> VariableRelation<'a> {
    pub fn check(&self, sub: &Substitution<'a>, w: &'a str) -> bool {
        match self {
            VariableRelation::LengthEquality {
                lhs: x,
                rhs_vars,
                c,
            } => {
                // if x is $U, it may not be in sub
                let lhs_len = if *x == UNIVERSE_CONSTANT {
                    w.len()
                } else {
                    sub.value(x).len()
                };
                // sum of the lengths of the rhs variables
                let rhs_len: usize = rhs_vars.iter().map(|var| sub.value(var).len()).sum();
                lhs_len == (rhs_len + c)
            }

            VariableRelation::Conjunction { constraints } => {
                // all constraints must hold
                for constraint in constraints {
                    if !constraint.check(sub, w) {
                        return false;
                    }
                }
                true
            }

            VariableRelation::Disjunction { constraints } => {
                // only one constraint must hold
                for constraint in constraints {
                    if constraint.check(sub, w) {
                        return true;
                    }
                }
                false
            }

            VariableRelation::Negation { inner } => !inner.check(sub, w),
            VariableRelation::VarPrefix {
                prefix_var: prefix,
                lhs,
            } => sub.value(lhs).starts_with(sub.value(prefix)),
            VariableRelation::ConstPrefix { lhs, prefix } => sub.value(lhs).starts_with(prefix),
        }
    }
}

/// A common interface for all word formula types
pub trait Formula: Sync + fmt::Display + fmt::Debug {
    fn free_vars(&self) -> Vec<&str>;
    fn check_substitution(&self, substitution: &Substitution, universe: &[&str]) -> bool;
    fn constraints(&'_ self) -> VariableRelation<'_>;
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
    fn check_substitution(&self, substitution: &Substitution, _universe: &[&str]) -> bool {
        let lhs_vec = vec![EquationContent::Variable(self.lhs_variable)];
        let lhs_sub = substitution.apply(&lhs_vec).join("");
        let rhs_sub = substitution.apply(&self.rhs).join("");

        lhs_sub == rhs_sub
    }

    fn constraints(&'_ self) -> VariableRelation<'_> {
        let rhs: Vec<_> = self
            .rhs
            .iter()
            .filter(|item| matches!(item, EquationContent::Variable(_)))
            .map(|var| {
                if let EquationContent::Variable(v) = var {
                    *v
                } else {
                    unreachable!()
                }
            })
            .sorted()
            .collect();

        // sum of the total lengths of all constants
        let sum_constant_lens: usize = self
            .rhs
            .iter()
            .filter(|item| matches!(item, EquationContent::Constant(_)))
            .map(|var| {
                if let EquationContent::Constant(v) = var {
                    (*v).len()
                } else {
                    unreachable!()
                }
            })
            .sum();

        let length_equality = VariableRelation::LengthEquality {
            lhs: self.lhs_variable,
            rhs_vars: rhs,
            c: sum_constant_lens,
        };

        match self.rhs[0] {
            EquationContent::Variable(var) => VariableRelation::Conjunction {
                constraints: vec![
                    VariableRelation::VarPrefix {
                        lhs: self.lhs_variable,
                        prefix_var: var,
                    },
                    length_equality,
                ],
            },
            EquationContent::Constant(constant) => VariableRelation::Conjunction {
                constraints: vec![
                    VariableRelation::ConstPrefix {
                        lhs: self.lhs_variable,
                        prefix: constant,
                    },
                    length_equality,
                ],
            },
        }
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

    fn constraints(&'_ self) -> VariableRelation<'_> {
        let lhs_constraints = self.lhs.constraints();
        let rhs_constraints = self.rhs.constraints();
        VariableRelation::Conjunction {
            constraints: vec![lhs_constraints, rhs_constraints],
        }
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

    fn constraints(&'_ self) -> VariableRelation<'_> {
        let lhs_constraints = self.lhs.constraints();
        let rhs_constraints = self.rhs.constraints();
        VariableRelation::Disjunction {
            constraints: vec![lhs_constraints, rhs_constraints],
        }
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

    fn constraints(&'_ self) -> VariableRelation<'_> {
        VariableRelation::Negation {
            inner: Box::from(self.inner.constraints()),
        }
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
        let mut altered_substitution =
            Substitution::from_vars(substitution.keys.clone(), substitution.universe_constant);
        altered_substitution.keys.push(self.bound_var);
        for val in &substitution.values {
            altered_substitution.values.push(*val);
        }
        altered_substitution.values.push("");
        let last_idx = altered_substitution.values.len() - 1;

        for factor in universe {
            altered_substitution.values[last_idx] = factor;
            let holds = self
                .inner
                .check_substitution(&altered_substitution, universe);
            if holds {
                return true;
            }
        }
        false
    }

    fn constraints(&'_ self) -> VariableRelation<'_> {
        //TODO: return a constraint that is the inner formula's constraint(s) with any references
        // to quantified variables removed
        VariableRelation::LengthEquality {
            lhs: UNIVERSE_CONSTANT,
            rhs_vars: vec![UNIVERSE_CONSTANT],
            c: 0,
        }
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
        let mut altered_substitution =
            Substitution::from_vars(substitution.keys.clone(), substitution.universe_constant);
        altered_substitution.keys.push(self.bound_var);
        for val in &substitution.values {
            altered_substitution.values.push(*val);
        }
        altered_substitution.values.push("");
        let last_idx = altered_substitution.values.len() - 1;

        for factor in universe {
            altered_substitution.values[last_idx] = factor;
            let holds = self
                .inner
                .check_substitution(&altered_substitution, universe);
            if holds {
                return false;
            }
        }
        true
    }

    fn constraints(&'_ self) -> VariableRelation<'_> {
        //TODO: return a constraint that is the inner formula's constraint(s) with any references
        // to quantified variables removed
        VariableRelation::LengthEquality {
            lhs: UNIVERSE_CONSTANT,
            rhs_vars: vec![UNIVERSE_CONSTANT],
            c: 0,
        }
    }
}

impl fmt::Display for UniversalFormula<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let _ = write!(f, "∀{x}: (", x = self.bound_var);
        let _ = self.inner.fmt(f);
        write!(f, ")")
    }
}
