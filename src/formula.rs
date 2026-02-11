use crate::strutils::CharOperator;
use crate::Substitution;
use itertools::Itertools;
use std::collections::HashSet;

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
    /// The constraint that guarantees nothing: [`VariableRelation::check`] will always return `true`
    NilConstraint,
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
            VariableRelation::NilConstraint => true,
        }
    }
}

#[derive(Debug)]
pub enum Quantifier {
    Universal,
    Existential,
}

#[derive(Debug)]
pub enum Formula<'a> {
    Equation {
        lhs: &'a str,
        rhs: Vec<EquationContent<'a>>,
    },
    Negation {
        inner: Box<Formula<'a>>,
    },
    Conjunction {
        fragments: Vec<Formula<'a>>,
    },
    Disjunction {
        fragments: Vec<Formula<'a>>,
    },
    Quantifier {
        inner: Box<Formula<'a>>,
        var: &'a str,
        quantifier: Quantifier,
    },
}

impl Formula<'_> {
    pub fn check_substitution(&self, w: &CharOperator, substitution: &Substitution) -> bool {
        match self {
            Formula::Equation { lhs, rhs } => {
                let lhs_value = substitution.value(lhs);
                let rhs_value = substitution.apply(rhs).join("");
                lhs_value == rhs_value
            }
            Formula::Negation { inner } => !inner.check_substitution(w, substitution),
            Formula::Conjunction { fragments } => {
                for fragment in fragments {
                    if !fragment.check_substitution(w, substitution) {
                        return false;
                    }
                }
                true
            }
            Formula::Disjunction { fragments } => {
                for fragment in fragments {
                    if fragment.check_substitution(w, substitution) {
                        return true;
                    }
                }
                false
            }
            Formula::Quantifier {
                inner,
                var,
                quantifier,
            } => {
                let mut altered_substitution = Substitution::from_vars(
                    substitution.keys.clone(),
                    substitution.universe_constant,
                );
                altered_substitution.keys.push(var);
                for val in &substitution.values {
                    altered_substitution.values.push(*val);
                }
                altered_substitution.values.push("");
                let last_idx = altered_substitution.values.len() - 1;
                match quantifier {
                    Quantifier::Universal => {
                        for factor in w.generate_factors() {
                            altered_substitution.values[last_idx] = factor;
                            let holds = inner.check_substitution(w, &altered_substitution);
                            if holds {
                                return false;
                            }
                        }
                        true
                    }
                    Quantifier::Existential => {
                        for factor in w.generate_factors() {
                            altered_substitution.values[last_idx] = factor;
                            let holds = inner.check_substitution(w, &altered_substitution);
                            if holds {
                                return true;
                            }
                        }
                        false
                    }
                }
            }
        }
    }

    pub fn free_vars(&self) -> Vec<&str> {
        match self {
            Formula::Equation { lhs, rhs } => {
                let mut free = vec![];
                if *lhs != UNIVERSE_CONSTANT {
                    free.push(*lhs);
                }

                for content in rhs {
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
            Formula::Negation { inner } => inner.free_vars(),
            Formula::Conjunction { fragments } => {
                let mut free = HashSet::new();
                for fragment in fragments {
                    for var in fragment.free_vars() {
                        free.insert(var);
                    }
                }
                free.into_iter().collect()
            }
            Formula::Disjunction { fragments } => {
                let mut free = HashSet::new();
                for fragment in fragments {
                    for var in fragment.free_vars() {
                        free.insert(var);
                    }
                }
                free.into_iter().collect()
            }
            Formula::Quantifier {
                var,
                quantifier: _quantifier,
                inner,
            } => {
                let mut free = inner.free_vars();
                let quantified_var_idx = free.iter().position(|v| v == var).unwrap();
                free.remove(quantified_var_idx);
                free
            }
        }
    }

    pub fn constraints(&'_ self) -> VariableRelation<'_> {
        match self {
            Formula::Equation { lhs, rhs } => {
                let rhs_variables: Vec<_> = rhs
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
                let sum_constant_lens: usize = rhs
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
                    lhs,
                    rhs_vars: rhs_variables,
                    c: sum_constant_lens,
                };

                match rhs[0] {
                    EquationContent::Variable(var) => VariableRelation::Conjunction {
                        constraints: vec![
                            VariableRelation::VarPrefix {
                                lhs,
                                prefix_var: var,
                            },
                            length_equality,
                        ],
                    },
                    EquationContent::Constant(constant) => VariableRelation::Conjunction {
                        constraints: vec![
                            VariableRelation::ConstPrefix {
                                lhs,
                                prefix: constant,
                            },
                            length_equality,
                        ],
                    },
                }
            }
            Formula::Negation { inner } => VariableRelation::Negation {
                inner: Box::new(inner.constraints()),
            },
            Formula::Conjunction { fragments } => {
                let mut constraints = Vec::with_capacity(fragments.len());
                for fragment in fragments {
                    constraints.push(fragment.constraints());
                }
                VariableRelation::Conjunction { constraints }
            }
            Formula::Disjunction { fragments } => {
                let mut constraints = Vec::with_capacity(fragments.len());
                for fragment in fragments {
                    constraints.push(fragment.constraints());
                }
                VariableRelation::Disjunction { constraints }
            }
            Formula::Quantifier { .. } => {
                //TODO: return a constraint that is the inner formula's constraint(s) with any references
                // to quantified variables removed
                VariableRelation::NilConstraint
            }
        }
    }
}
