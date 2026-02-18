use crate::strutils::CharOperator;
use crate::{print_solutions, strutils, Substitution};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

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
                let mut altered_substitution =
                    Substitution::from_vars(&substitution.keys, substitution.universe_constant);
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
                    // solutions_for_equation(lhs_value, lhs, rhs);
                    let lhs_chars = CharOperator::new(lhs);
                    let lhs_factors = lhs_chars.generate_factors();
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

struct EquationVariable<'a> {
    name: &'a str,
    follows_const: Option<&'a str>,
}

enum PatternAssignmentAtom<'a> {
    VariableAssignment { var: &'a str, value: &'a str },
    ConstantAssignment { value: &'a str, pos: usize },
}

/// It is assumed that `rhs` starts and ends with a variable, otherwise the string `lhs_assignment`
/// can be trimmed according to the surrounding constants on the rhs
fn solutions_for_equation<'a>(
    lhs_assignment: &'a CharOperator,
    lhs_var: &str,
    rhs: &'a [EquationContent],
) -> Vec<Substitution<'a>> {
    // string constants in the order in which they appear in the formula
    let mut constants = Vec::new();
    let mut const_positions = HashMap::new();
    let mut formula_vars = Vec::new();
    for component in rhs {
        match component {
            EquationContent::Variable(var) => {
                formula_vars.push(*var);
            }
            EquationContent::Constant(val) => {
                constants.push(*val);
                let positions = lhs_assignment.find(val);
                if positions.is_empty() {
                    // lhs assignment doesn't contain a required constant in the equation rhs
                    return vec![];
                } else {
                    const_positions.insert(*val, positions);
                }
            }
        }
    }

    // TODO These loops can probably be combined
    let mut variable_sequences: Vec<Vec<&str>> = Vec::new();
    let mut var_seq_idx = 0;
    for component in rhs {
        match component {
            EquationContent::Variable(var) => {
                if var_seq_idx >= variable_sequences.len() {
                    variable_sequences.push(Vec::new());
                }
                variable_sequences[var_seq_idx].push(*var);
            }
            EquationContent::Constant(val) => {
                var_seq_idx += 1;
                if !variable_sequences[variable_sequences.len() - 1].is_empty() {
                    variable_sequences.push(Vec::new());
                }
            }
        }
    }
    // Case: no constants on rhs
    if constants.is_empty() {
        // all possible 'partition' assignments to rhs vars are satisfying, unless a variable appears multiple times
        let mut assignments = Vec::new();
        let assignment_values = partition_string(lhs_assignment, formula_vars.len());
        for assignment in assignment_values {
            let mut substitution = Substitution::from_vars(&formula_vars, lhs_assignment.as_str());
            for (i, var) in formula_vars.iter().enumerate() {
                substitution.insert(var, assignment[i]);
            }
            // TODO - is there a better way than this?
            let result_str = substitution.apply(rhs);
            let result_str = result_str.join("");
            if result_str == lhs_assignment.as_str() {
                assignments.push(substitution);
            }
        }
        return assignments;
    }

    let mut partial_assignments = Vec::new();
    // Contains the index and length of the constant that follows the assignments in partial_assignments
    let mut corresponding_constant_info = Vec::new();

    let mut constants_analysed = 0;

    // positions of first constant
    let constant = constants[0];
    let first_const_positions = &const_positions[constant];
    for &const_position in first_const_positions {
        let prefix_operator = CharOperator::new(lhs_assignment.substring(0, const_position));

        // assign the variables that came before this constant
        let var_sequence = &variable_sequences[0];
        let assignment_values = partition_string(&prefix_operator, var_sequence.len());
        for values in assignment_values {
            let mut partial_assignment =
                Substitution::from_vars(var_sequence, lhs_assignment.as_str());
            for (i, values) in values.iter().enumerate() {
                partial_assignment.values.push(values);
            }
            partial_assignments.push(partial_assignment);
            corresponding_constant_info.push((const_position, strutils::count_chars(constant)));
        }
    }
    constants_analysed += 1;

    // TODO - now we need to deal with variables that come after the first constant

    if constants_analysed == constants.len() {
        // all constants dealt with
        // now see which variables haven't been assigned to yet. This will be the same across all
        //  assignments in partial_assignments
        let mut remaining_vars = formula_vars.clone();
        for var in &partial_assignments[0].keys {
            let idx = remaining_vars.iter().find_position(|&v| v == var);
            if let Some((idx, _)) = idx {
                remaining_vars.swap_remove(idx);
            }
        }
        println!("Unassigned vars: {:?}", remaining_vars);
        let mut satisfying_assignments = Vec::new();
        for (i, partial_assignment) in partial_assignments.iter().enumerate() {
            // Now we split the remaining lhs assignment string among the remaining variables, using
            // the additional array to discover where each partial assignment 'ends' in the lhs string
            let (const_idx, const_len) = corresponding_constant_info[i];
            let remaining_string =
                lhs_assignment.substring(const_idx + const_len, lhs_assignment.len());
            let remaining_string = CharOperator::new(remaining_string);

            let remaining_assignment_values =
                partition_string(&remaining_string, remaining_vars.len());
            for values in remaining_assignment_values {
                let mut complete_assignment =
                    Substitution::from_vars(&formula_vars, lhs_assignment.as_str());
                for j in 0..partial_assignment.keys.len() {
                    complete_assignment
                        .insert(partial_assignment.keys[j], partial_assignment.values[j]);
                }

                for j in 0..values.len() {
                    complete_assignment.insert(remaining_vars[j], values[j]);
                }

                // TODO - is this necessary/can it be improved?
                if complete_assignment.apply(rhs).join("") == lhs_assignment.as_str() {
                    satisfying_assignments.push(complete_assignment);
                }
            }
        }
        return satisfying_assignments;
    }

    if constants.len() > 1 {
        todo!("Cases where there are multiple constants")
    }

    println!("assignments: {:?}", partial_assignments);

    // Now check these partial assignments, they may not all be correct across all 'shortcuts' between constants
    // TODO - is there a better 'general' way of doing this?
    let mut satisfying_assignments = Vec::with_capacity(partial_assignments.len());
    for substitution in partial_assignments {
        let result_str = substitution.apply(rhs);
        let result_str = result_str.join("");
        if result_str == lhs_assignment.as_str() {
            satisfying_assignments.push(substitution);
        }
    }
    satisfying_assignments
}

#[test]
fn test_new_eq_solver() {
    let lhs_assignment = CharOperator::new("abcdeafa");
    let sols = solutions_for_equation(
        &lhs_assignment,
        "x",
        &[
            EquationContent::Variable("q"),
            EquationContent::Variable("w"),
            EquationContent::Variable("r"),
            EquationContent::Constant("a"),
            EquationContent::Variable("s"),
        ],
    );
    print_solutions(&sols, lhs_assignment.as_str());
}

fn partition_string<'a>(string: &CharOperator<'a>, num_vars: usize) -> Vec<Vec<&'a str>> {
    let partition_positions = (0..string.len() + 1).combinations_with_replacement(num_vars - 1);
    let mut substitutions = Vec::with_capacity(partition_positions.try_len().unwrap());
    for partition_position in partition_positions {
        let mut partition_position = partition_position;
        partition_position.insert(0, 0);
        partition_position.push(string.len());
        let sub = if string.len() == 0 {
            vec![""; num_vars]
        } else {
            string.multi_substring(&partition_position)
        };
        substitutions.push(sub);
    }
    substitutions
}

#[test]
fn test_partitions() {
    let s = CharOperator::new("abcde");
    let n = 3;
    let partitions = partition_string(&s, n);
    println!("{:?}", partitions);
    assert_eq!(partitions.len(), 21);
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
        let mut sub = Substitution::from_vars(&formula.free_vars(), "");
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
    print_solutions(&sols, "bbabab");
}
