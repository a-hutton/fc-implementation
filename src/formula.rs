use crate::strutils::CharOperator;
use crate::{print_solutions, strutils, Substitution};
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

impl<'a> Formula<'a> {
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

    pub fn all_solutions(&'a self, w: &'a CharOperator) -> Vec<Substitution<'a>> {
        match self {
            Formula::Equation { lhs, rhs } => {
                // TODO - special case for $U
                let mut substitutions = Vec::new();
                for lhs_value in w.generate_factors() {
                    // For each possible value for the lhs variable
                    let subs_for_val = rhs_assignments(lhs_value, lhs, rhs, self);
                    substitutions.extend(subs_for_val);
                }
                substitutions
            }
            Formula::Negation { inner } => {
                todo!()
            }
            Formula::Conjunction { fragments } => {
                let mut satisfying_assignments = fragments[0].all_solutions(w);
                let mut i = 0;
                while i < satisfying_assignments.len() {
                    for fragment in &fragments[1..] {
                        if !fragment.check_substitution(w, &satisfying_assignments[i]) {
                            satisfying_assignments.swap_remove(i);
                        }
                    }
                    i += 1;
                }
                satisfying_assignments
            }
            Formula::Disjunction { fragments } => {
                let mut satisfying_assignment = HashSet::new();
                for fragment in fragments {
                    let fragment_sat_assignments = fragment.all_solutions(w);
                    for assignment in fragment_sat_assignments {
                        satisfying_assignment.insert(assignment);
                    }
                }
                satisfying_assignment.into_iter().collect_vec()
            }
            Formula::Quantifier {
                var,
                quantifier: _quantifier,
                inner,
            } => {
                todo!()
            }
        }
    }
}

fn rhs_assignments<'a>(
    lhs_value: &'a str,
    lhs_var: &'a str,
    rhs: &Vec<EquationContent<'a>>,
    formula: &'a Formula<'_>,
) -> Vec<Substitution<'a>> {
    let lhs_chars = strutils::CharOperator::new(lhs_value);

    // TODO - filter out duplicates.... somehow
    // TODO - filter out obvious fails as consts must be in sequence
    let mut pattern_positions = Vec::with_capacity(rhs.len());
    for p in rhs.iter() {
        match p {
            EquationContent::Variable(_) => {
                pattern_positions.push(PatternPositions::Variable(None))
            }
            EquationContent::Constant(val) => {
                let const_positions = lhs_chars.find(val);
                if const_positions.is_empty() {
                    // required const not found
                    return vec![];
                }

                let const_positions = const_positions
                    .into_iter()
                    .map(|i| (i, i + val.len()))
                    .collect_vec();
                pattern_positions.push(PatternPositions::Constant(const_positions));
            }
        }
    }

    let start_position = LookBehind::Constant(vec![(0, 0)]);
    let end_position = vec![(lhs_chars.len(), lhs_chars.len())];
    for i in 0..rhs.len() {
        let is_start = i == 0;
        let is_final = i == rhs.len() - 1;
        // Look ahead and look behind are either a single const, or a group of variables until the next const.

        // look behind
        let mut look_behinds = Vec::with_capacity(i);
        if is_start {
            look_behinds.push(start_position.clone());
        } else {
            let mut prev = &pattern_positions[i - 1];
            match &prev {
                PatternPositions::Variable(_) => {
                    let mut is_var = true;
                    let mut j = 1;
                    while is_var {
                        match prev {
                            PatternPositions::Variable(_) => {
                                if j > i {
                                    break;
                                }
                                let prev_look_behind = LookBehind::Variables(vec![]);
                                look_behinds.push(prev_look_behind);
                                prev = &pattern_positions[i - j];
                                j += 1;
                            }
                            PatternPositions::Constant(_) => {
                                is_var = false;
                            }
                        }
                    }
                }
                PatternPositions::Constant(c) => {
                    let c = LookBehind::Constant(c.clone());
                    look_behinds.push(c);
                }
            }
        }

        // look ahead
        let mut look_ahead = None;
        if is_final {
            look_ahead = Some(&end_position);
        } else {
            let next = &pattern_positions[i + 1];
            match &next {
                PatternPositions::Variable(_) => {}
                PatternPositions::Constant(positions) => {
                    look_ahead = Some(positions);
                }
            }
        }
        let pattern_element = &rhs[i];

        match pattern_element {
            EquationContent::Variable(_) => {
                let mut positions = Vec::new();
                // lookbehind (i,j) - _j_ is possible start location
                // lookahead (i,j) - _i_ is possible end location
                for b in look_behinds {
                    match b {
                        LookBehind::Variables(_) => {
                            todo!()
                        }
                        LookBehind::Constant(val_positions) => {
                            for (_, j) in val_positions {
                                for &a in look_ahead.iter() {
                                    for (i, _) in a {
                                        positions.push((j, *i));
                                    }
                                }
                            }
                        }
                    }
                }

                pattern_positions[i] = PatternPositions::Variable(Some(positions))
            }
            EquationContent::Constant(_) => {}
        }
    }

    let pattern_positions = pattern_positions.iter().map(|p| match p {
        PatternPositions::Variable(var) => match var {
            None => {
                panic!()
            }
            Some(var_positions) => var_positions
                .iter()
                .filter(|(i, j)| i <= j)
                .map(|(i, j)| lhs_chars.substring(*i, *j))
                .collect_vec(),
        },
        PatternPositions::Constant(vals) => vals
            .iter()
            .map(|(i, j)| lhs_chars.substring(*i, *j))
            .collect_vec(),
    });

    let position_combinations = pattern_positions.multi_cartesian_product();
    let mut substitutions = Vec::with_capacity(position_combinations.try_len().unwrap());
    let mut combo_set = HashSet::new();
    for combination in position_combinations {
        // This is naive duplicate filter
        if combo_set.contains(&combination) {
            continue;
        }
        combo_set.insert(combination.clone());

        let free_vars = formula.free_vars();
        let mut sub = Substitution::from_vars(formula.free_vars(), "");
        // TODO?
        // sub.insert(UNIVERSE_CONSTANT, "");

        sub.insert(lhs_var, lhs_value);
        for (i, pattern_element) in rhs.iter().enumerate() {
            let assignment = combination[i];
            match pattern_element {
                EquationContent::Variable(var) => {
                    sub.insert(*var, assignment);
                }
                EquationContent::Constant(val) => {
                    assert_eq!(*val, assignment)
                }
            }
        }
        if formula.check_substitution(&lhs_chars, &sub) {
            substitutions.push(sub);
        }
    }

    substitutions
}

enum PatternPositions {
    Variable(Option<Vec<(usize, usize)>>),
    Constant(Vec<(usize, usize)>),
}

#[derive(Clone)]
enum LookBehind {
    Variables(Vec<Vec<(usize, usize)>>),
    Constant(Vec<(usize, usize)>),
}

#[test]
fn test_new_method() {
    let eq = Formula::Equation {
        lhs: "x",
        rhs: vec![
            EquationContent::Variable("y"),
            EquationContent::Constant("a"),
            EquationContent::Variable("z"),
            EquationContent::Constant("b"),
            EquationContent::Variable("q"),
        ],
    };
    let w = CharOperator::new("bbabab");
    let sols = eq.all_solutions(&w);
    print_solutions(&sols, "bbabab", vec!["x", "y", "z", "q"]);
}
