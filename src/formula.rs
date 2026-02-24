use crate::strutils::CharOperator;
use crate::{print_solutions, Substitution};
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
                // TODO - special case for $U ?
                // Ensure that the 'rhs' starts and ends with a variable
                // We can use the prefix and suffix extracted here to do a quick check on potential
                // assignments to the lhs variable
                let (trimmed_rhs, required_prefix, required_suffix) = trim_equation_rhs(rhs);
                if *lhs == UNIVERSE_CONSTANT {
                    if !w.as_str().starts_with(&required_prefix)
                        || !w.as_str().ends_with(&required_suffix)
                    {
                        return vec![];
                    }
                    solutions_for_equation(w.as_str(), lhs, trimmed_rhs)
                } else {
                    let mut substitutions = Vec::new();
                    for lhs_value in w.generate_factors() {
                        if !lhs_value.starts_with(&required_prefix)
                            || !lhs_value.ends_with(&required_suffix)
                        {
                            continue;
                        }
                        let trimmed_lhs_value = &lhs_value
                            [required_prefix.len()..lhs_value.len() - required_suffix.len()];
                        let assignments =
                            solutions_for_equation(trimmed_lhs_value, lhs, trimmed_rhs);
                        let assignments = assignments
                            .into_iter()
                            .map(|mut sub| {
                                sub.insert(lhs, lhs_value);
                                sub
                            })
                            .filter(|sub| sub.apply(rhs).join("") == lhs_value)
                            .collect_vec();
                        substitutions.extend(assignments);
                    }
                    substitutions
                }
            }
            Formula::Negation { inner: _inner } => {
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
                var: _,
                quantifier: _quantifier,
                inner: _,
            } => {
                todo!()
            }
        }
    }
}

fn trim_equation_rhs<'a, 'b>(
    rhs: &'a [EquationContent<'b>],
) -> (&'a [EquationContent<'b>], String, String) {
    let mut trimmed_rhs = rhs;
    let first = &trimmed_rhs[0];
    let mut prefix = String::new();
    if let EquationContent::Constant(val) = first {
        trimmed_rhs = &trimmed_rhs[1..];
        prefix = String::from(*val);
    }
    if trimmed_rhs.is_empty() {
        return (trimmed_rhs, prefix, String::new());
    }

    let last = &trimmed_rhs[trimmed_rhs.len() - 1];
    let mut suffix = String::new();
    if let EquationContent::Constant(val) = last {
        trimmed_rhs = &trimmed_rhs[..trimmed_rhs.len() - 1];
        suffix = String::from(*val);
    }
    if trimmed_rhs.is_empty() {
        return (trimmed_rhs, prefix, String::new());
    }

    if matches!(&trimmed_rhs[0], EquationContent::Constant(_))
        || matches!(
            &trimmed_rhs[trimmed_rhs.len() - 1],
            EquationContent::Constant(_)
        )
    {
        let (inner_trim, inner_prefix, inner_suffix) = trim_equation_rhs(trimmed_rhs);
        trimmed_rhs = inner_trim;
        prefix += &*inner_prefix;
        suffix = inner_suffix + &*suffix;
    }

    (trimmed_rhs, prefix, suffix)
}

#[test]
fn test_trim_equation_rhs() {
    let rhs = [
        EquationContent::Constant("aaa"),
        EquationContent::Constant("bbb"),
        EquationContent::Variable("x"),
        EquationContent::Constant("middle"),
        EquationContent::Variable("y"),
        EquationContent::Constant("zz"),
    ];
    let (trimmed_rhs, prefix, suffix) = trim_equation_rhs(&rhs);
    println!("{:?}", trimmed_rhs);
    assert_eq!(prefix, "aaabbb");
    assert_eq!(suffix, "zz");
    assert_eq!(trimmed_rhs.len(), 3);
}

/// It is assumed that `rhs` starts and ends with a variable, otherwise the string `lhs_assignment`
/// can be trimmed according to the surrounding constants on the rhs
fn solutions_for_equation<'a>(
    lhs_assignment: &'a str,
    lhs_var: &'a str,
    rhs: &'a [EquationContent],
) -> Vec<Substitution<'a>> {
    let mut variable_sequences_end_indices = Vec::new();
    let mut constants = Vec::new();
    let mut formula_vars = Vec::with_capacity(rhs.len());
    for component in rhs {
        match component {
            EquationContent::Variable(var) => {
                formula_vars.push(*var);
            }
            EquationContent::Constant(val) => {
                variable_sequences_end_indices.push(formula_vars.len());
                constants.push(*val);
            }
        }
    }
    variable_sequences_end_indices.push(formula_vars.len());

    // Case: no variables on rhs (hit when lhs is universe constant)
    if formula_vars.is_empty() {
        // Not an empty 'set' of assignments, the set containing an empty assignment
        return vec![Substitution::from_vars(&[], lhs_assignment)];
    }

    // Case: no constants on rhs
    if constants.is_empty() {
        // all possible 'partition' assignments to rhs vars are satisfying, unless a variable appears multiple times
        let assignments =
            equation_subsequence_solutions(&formula_vars, lhs_assignment).collect_vec();
        return assignments;
    }

    let combined_assignments = equation_assignments(
        &formula_vars,
        &variable_sequences_end_indices,
        &constants,
        lhs_assignment,
    );
    let satisfying_assignments = combined_assignments
        .into_iter()
        .filter(|sub| {
            let applied_str = sub.apply(rhs).join("");
            applied_str == lhs_assignment
        })
        .collect_vec();
    satisfying_assignments
}

/// Returns assignments that satisfy the equation formed by taking the first variable sequence in
/// defined by `variables` and end indices in `variable_sequence_indices`, then the first constant
/// in `combined_constants`, and so on, alternating. Assumes pattern starts and ends with a variable
fn equation_assignments<'a>(
    variables: &[&'a str],
    variable_sequence_indices: &[usize],
    constants: &Vec<&str>,
    lhs_assignment: &'a str,
) -> Vec<Substitution<'a>> {
    let mut const_positions = Vec::with_capacity(constants.len());
    let lhs_chars = CharOperator::new(lhs_assignment);
    for constant in constants {
        let positions = lhs_chars.find(constant);
        const_positions.push((constant.len(), positions));
    }

    let mut constituent_partial_assignments = Vec::with_capacity(variable_sequence_indices.len());

    let mut var_sequence_start_idx = 0;
    for (i, &var_sequence_end_idx) in variable_sequence_indices.iter().enumerate() {
        let possible_start_indices = if i == 0 {
            vec![0]
        } else {
            let (len, indices) = &const_positions[i - 1];
            indices.iter().map(|i| i + len).collect_vec()
        };
        let var_sequence = &variables[var_sequence_start_idx..var_sequence_end_idx];
        let possible_end_indices = if i == variable_sequence_indices.len() - 1 {
            &vec![lhs_assignment.len()]
        } else {
            let (_, indices) = &const_positions[i];
            indices
        };

        let partial_assignments = var_sequence_partial_assignment(
            var_sequence,
            &lhs_chars,
            &possible_start_indices,
            possible_end_indices,
        );

        constituent_partial_assignments.push(partial_assignments);
        var_sequence_start_idx = var_sequence_end_idx;
    }

    let combined_assignments = constituent_partial_assignments.into_iter().reduce(|a, b| {
        let mut subs = Vec::with_capacity(a.len() * b.len());
        for assignment_a in &a {
            for assignment_b in &b {
                let res = Substitution::join(assignment_a, assignment_b);
                if let Ok(s) = res {
                    subs.push(s);
                } else {
                    println!(
                        "Didn't join assignments - conflicting values: {}",
                        res.unwrap_err()
                    )
                }
            }
        }
        subs
    });
    combined_assignments.unwrap_or_default()
}

/// All assignments to a variable sequence across possible combinations of start/end indices
fn var_sequence_partial_assignment<'a>(
    var_sequence: &[&'a str],
    lhs_assignment: &CharOperator<'a>,
    possible_start_indices: &Vec<usize>,
    possible_end_indices: &Vec<usize>,
) -> Vec<Substitution<'a>> {
    let mut partial_assignments = Vec::new();
    for &start in possible_start_indices {
        for &end in possible_end_indices {
            if start <= end {
                let target_substring = lhs_assignment.substring(start, end);
                let substitutions =
                    equation_subsequence_solutions(var_sequence, target_substring).collect_vec();
                partial_assignments.extend(substitutions);
            }
        }
    }
    partial_assignments
}

/// Finds partial assignments for the variables in `var_sequence` to make the substring
fn equation_subsequence_solutions<'a>(
    var_sequence: &[&'a str],
    lhs_substring: &'a str,
) -> impl Iterator<Item = Substitution<'a>> {
    let lhs_chars = CharOperator::new(lhs_substring);
    let mut partitioned_values = partition_string(&lhs_chars, var_sequence.len()).into_iter();
    std::iter::from_fn(move || {
        loop {
            let values = partitioned_values.next();
            if let Some(values) = values {
                let mut substitution = Substitution::from_vars(var_sequence, lhs_substring);
                for i in 0..values.len() {
                    substitution.insert(var_sequence[i], values[i]);
                }
                let content = var_sequence
                    .iter()
                    .map(|s| EquationContent::Variable(s))
                    .collect_vec();
                if substitution.apply(&content).join("") == lhs_substring {
                    return Some(substitution);
                }
            } else {
                return None;
            }
        }
    })
}

#[test]
fn test_new_eq_solver() {
    let lhs_assignment = "abcdeafa";
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
    print_solutions(&sols, lhs_assignment);

    let lhs_assignment = "cdababe";
    let sols = solutions_for_equation(
        &lhs_assignment,
        "q",
        &[
            EquationContent::Variable("r"),
            EquationContent::Constant("a"),
            EquationContent::Variable("s"),
            EquationContent::Variable("t"),
            EquationContent::Constant("b"),
            EquationContent::Variable("u"),
        ],
    );
    print_solutions(&sols, lhs_assignment);
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
