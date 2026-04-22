use crate::assignment::Assignment;
use crate::print_assignments;
use crate::strutils::CharOperator;
use itertools::Itertools;
use std::collections::HashSet;

pub const UNIVERSE_CONSTANT: &str = "$U";

/// Represents values that can appear in an 'atomic' word equation - either a variable with a name,
/// or a string constant
#[derive(Debug, Hash)]
pub enum EquationContent<'a> {
    Variable(&'a str),
    Constant(&'a str),
    UniverseConstant,
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
    pub fn is_satisfying_assignment(&self, w: &CharOperator, assignment: &Assignment) -> bool {
        match self {
            Formula::Equation { lhs, rhs } => {
                // TODO - must check that all values are in the universe
                let lhs_value = assignment.value(lhs);
                let rhs_value = assignment.apply(rhs).join("");
                lhs_value == rhs_value
            }
            Formula::Negation { inner } => !inner.is_satisfying_assignment(w, assignment),
            Formula::Conjunction { fragments } => {
                for fragment in fragments {
                    if !fragment.is_satisfying_assignment(w, assignment) {
                        return false;
                    }
                }
                true
            }
            Formula::Disjunction { fragments } => {
                for fragment in fragments {
                    if fragment.is_satisfying_assignment(w, assignment) {
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
                let mut altered_assignment = assignment.clone();
                match quantifier {
                    Quantifier::Universal => {
                        for factor in w.generate_factors() {
                            altered_assignment.insert(var, factor);
                            let holds = inner.is_satisfying_assignment(w, &altered_assignment);
                            if holds {
                                return false;
                            }
                        }
                        true
                    }
                    Quantifier::Existential => {
                        for factor in w.generate_factors() {
                            altered_assignment.insert(var, factor);
                            let holds = inner.is_satisfying_assignment(w, &altered_assignment);
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

    pub fn all_solutions(&'a self, w: &'a CharOperator) -> Vec<Assignment<'a>> {
        match self {
            Formula::Equation { lhs, rhs } => {
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
                    let trimmed_universe = &w.as_str()
                        [required_prefix.len()..w.as_str().len() - required_suffix.len()];
                    partial_assignments_for_equation(trimmed_universe, trimmed_rhs, w.as_str())
                } else {
                    let mut all_assignments = Vec::new();
                    for lhs_value in w.generate_factors() {
                        if lhs_value.len() < required_prefix.len() + required_suffix.len() {
                            continue;
                        }
                        if !lhs_value.starts_with(&required_prefix)
                            || !lhs_value.ends_with(&required_suffix)
                        {
                            continue;
                        }
                        let trimmed_lhs_value = &lhs_value
                            [required_prefix.len()..lhs_value.len() - required_suffix.len()];
                        let assignments = partial_assignments_for_equation(
                            trimmed_lhs_value,
                            trimmed_rhs,
                            w.as_str(),
                        );
                        let assignments_for_lhs_value = assignments
                            .into_iter()
                            .map(|mut assignment| {
                                assignment.insert(lhs, lhs_value);
                                assignment
                            })
                            .filter(|assignment| assignment.apply(rhs).join("") == lhs_value)
                            .collect_vec();
                        all_assignments.extend(assignments_for_lhs_value);
                    }
                    all_assignments
                }
            }
            Formula::Negation { inner } => {
                println!("Using negation without a guard (B && ¬A) may take exponentially long");
                // This is brute force
                let universe = w.generate_factors();
                let free_vars = inner.free_vars();
                let values =
                    std::iter::repeat_n(universe, free_vars.len()).multi_cartesian_product();
                values
                    .map(|values| {
                        let mut ass = Assignment::from_vars(&free_vars, w.as_str());
                        for i in 0..free_vars.len() {
                            ass.insert(free_vars[i], values[i])
                        }
                        ass
                    })
                    .filter(|assignment| !inner.is_satisfying_assignment(w, assignment))
                    .collect_vec()
            }
            Formula::Conjunction { fragments } => {
                let first_assignments = fragments[0].all_solutions(w);

                // Find the variables that appear in fragments[1..], but not fragments[0]
                let first_fragment_vars = fragments[0].free_vars();
                let mut unseen_vars = HashSet::new();
                for fragment in &fragments[1..] {
                    let fragment_vars = fragment.free_vars();
                    for var in fragment_vars {
                        if !first_fragment_vars.contains(&var) {
                            unseen_vars.insert(var);
                        }
                    }
                }
                let unseen_vars = unseen_vars.into_iter().collect_vec();
                let universe = w.generate_factors();

                if unseen_vars.is_empty() {
                    first_assignments
                        .into_iter()
                        .filter(|assignment| {
                            for fragment in &fragments[1..] {
                                if !fragment.is_satisfying_assignment(w, assignment) {
                                    return false;
                                }
                            }
                            true
                        })
                        .collect_vec()
                } else {
                    let mut satisfying_assignments = Vec::new();
                    for assignment in &first_assignments {
                        let modified_assignments =
                            Assignment::extend_with_universe(assignment, &unseen_vars, &universe);

                        let satisfying_modified_assignments = modified_assignments
                            .into_iter()
                            .filter(|assignment| {
                                for fragment in &fragments[1..] {
                                    if !fragment.is_satisfying_assignment(w, assignment) {
                                        return false;
                                    }
                                }
                                true
                            })
                            .collect_vec();
                        satisfying_assignments.extend(satisfying_modified_assignments);
                    }
                    satisfying_assignments
                }
            }
            Formula::Disjunction { fragments } => {
                let free_vars: HashSet<&str> = HashSet::from_iter(self.free_vars());
                let mut satisfying_assignments = HashSet::new();
                let universe = w.generate_factors();
                for fragment in fragments {
                    let fragment_vars = HashSet::from_iter(fragment.free_vars());
                    let missing_fragment_vars =
                        free_vars.difference(&fragment_vars).copied().collect_vec();
                    let fragment_sat_assignments = fragment.all_solutions(w);
                    let mut extended_fragment_sat_assignments = Vec::new();
                    for assignment in fragment_sat_assignments {
                        if missing_fragment_vars.is_empty() {
                            extended_fragment_sat_assignments.push(assignment);
                        } else {
                            let mutated_assignments = Assignment::extend_with_universe(
                                &assignment,
                                &missing_fragment_vars,
                                &universe,
                            );
                            extended_fragment_sat_assignments.extend(mutated_assignments);
                        }
                    }

                    for assignment in extended_fragment_sat_assignments {
                        satisfying_assignments.insert(assignment);
                    }
                }
                satisfying_assignments.into_iter().collect_vec()
            }
            Formula::Quantifier {
                var,
                quantifier,
                inner,
            } => match quantifier {
                Quantifier::Universal => {
                    let inner_solutions = inner.all_solutions(w);
                    let mut var_values = HashSet::new();
                    for solution in &inner_solutions {
                        let val = solution.value(var);
                        var_values.insert(val);
                    }
                    if var_values.len() != w.generate_factors().len() {
                        return vec![];
                    }
                    inner_solutions
                        .into_iter()
                        .map(|mut ass| {
                            ass.remove_variable(var);
                            ass
                        })
                        .collect_vec()
                }
                Quantifier::Existential => {
                    let inner_solutions = inner.all_solutions(w);
                    if inner_solutions.is_empty() {
                        return vec![];
                    }
                    // remove the quantified variable
                    let mut new_solutions = HashSet::with_capacity(inner_solutions.len());
                    for mut ass in inner_solutions {
                        ass.remove_variable(var);
                        new_solutions.insert(ass);
                    }
                    new_solutions.into_iter().collect_vec()
                }
            },
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
fn partial_assignments_for_equation<'a>(
    lhs_assignment: &'a str,
    rhs: &'a [EquationContent],
    universe_word: &'a str,
) -> Vec<Assignment<'a>> {
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
            EquationContent::UniverseConstant => {
                constants.push(universe_word);
            }
        }
    }
    variable_sequences_end_indices.push(formula_vars.len());

    // Case: no variables on rhs (hit when lhs is universe constant)
    if formula_vars.is_empty() {
        // Not an empty 'set' of assignments, the set containing an empty assignment
        return vec![Assignment::from_vars(&[], universe_word)];
    }

    // Case: no constants on rhs
    if constants.is_empty() {
        // all possible 'partition' assignments to rhs vars are satisfying, unless a variable appears multiple times
        let assignments =
            equation_subsequence_partial_assignments(&formula_vars, lhs_assignment, universe_word)
                .collect_vec();
        return assignments;
    }

    let combined_assignments = equation_assignments(
        &formula_vars,
        &variable_sequences_end_indices,
        &constants,
        lhs_assignment,
        universe_word,
    );
    let satisfying_assignments = combined_assignments
        .into_iter()
        .filter(|assignment| {
            let applied_str = assignment.apply(rhs).join("");
            applied_str == lhs_assignment
        })
        .collect_vec();
    satisfying_assignments
}

/// Returns assignments that satisfy the equation formed by taking the first variable sequence in
/// defined by `variables` and end indices in `variable_sequence_indices`, then the first constant
/// in `combined_constants`, and so on, alternating. Assumes pattern starts and ends with a variable.
///
/// E.g. x = abc "mn" d
/// ```
/// variables = &["a", "b", "c", "d"];
/// variable_sequence_indices = &[2,3];
/// ```
/// ## Parameters
/// - `variables`: The variables in the equation rhs, in the order they appear
/// - `variable_sequence_indices`: Indices of `variables` showing the _end_ index of each variable
///   sequence in the rhs
fn equation_assignments<'a>(
    variables: &[&'a str],
    variable_sequence_indices: &[usize],
    constants: &Vec<&str>,
    lhs_assignment: &'a str,
    universe_word: &'a str,
) -> Vec<Assignment<'a>> {
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
            universe_word,
        );

        constituent_partial_assignments.push(partial_assignments);
        var_sequence_start_idx = var_sequence_end_idx;
    }

    let combined_assignments = constituent_partial_assignments.into_iter().reduce(|a, b| {
        let mut assignments = Vec::with_capacity(a.len() * b.len());
        for assignment_a in &a {
            for assignment_b in &b {
                let res = Assignment::join(assignment_a, assignment_b);
                if let Ok(s) = res {
                    assignments.push(s);
                } else {
                    println!(
                        "Didn't join assignments - conflicting values: {}",
                        res.unwrap_err()
                    )
                }
            }
        }
        assignments
    });
    combined_assignments.unwrap_or_default()
}

/// All assignments to a variable sequence across possible combinations of start/end indices
/// If it is known that for this sequence of variables V, there are a limited number of possibilities
/// for where σ(V) falls within the lhs assignment, they can be passed as possible start/end indices here
fn var_sequence_partial_assignment<'a>(
    var_sequence: &[&'a str],
    lhs_assignment: &CharOperator<'a>,
    possible_start_indices: &Vec<usize>,
    possible_end_indices: &Vec<usize>,
    universe_word: &'a str,
) -> Vec<Assignment<'a>> {
    let mut partial_assignments = Vec::new();
    for &start in possible_start_indices {
        for &end in possible_end_indices {
            if start <= end {
                let target_substring = lhs_assignment.substring(start, end);
                let assignments = equation_subsequence_partial_assignments(
                    var_sequence,
                    target_substring,
                    universe_word,
                )
                .collect_vec();
                partial_assignments.extend(assignments);
            }
        }
    }
    partial_assignments
}

/// Finds partial assignments for the variables in `var_sequence` to make the substring
fn equation_subsequence_partial_assignments<'a>(
    var_sequence: &[&'a str],
    lhs_substring: &'a str,
    universe_word: &'a str,
) -> impl Iterator<Item = Assignment<'a>> {
    let lhs_chars = CharOperator::new(lhs_substring);
    let mut partitioned_values = partition_string(&lhs_chars, var_sequence.len()).into_iter();
    std::iter::from_fn(move || {
        loop {
            let values = partitioned_values.next();
            if let Some(values) = values {
                let mut assignment = Assignment::from_vars(var_sequence, universe_word);
                for i in 0..values.len() {
                    assignment.insert(var_sequence[i], values[i]);
                }
                let content = var_sequence
                    .iter()
                    .map(|s| EquationContent::Variable(s))
                    .collect_vec();
                if assignment.apply(&content).join("") == lhs_substring {
                    return Some(assignment);
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
    let sols = partial_assignments_for_equation(
        &lhs_assignment,
        &[
            EquationContent::Variable("q"),
            EquationContent::Variable("w"),
            EquationContent::Variable("r"),
            EquationContent::Constant("a"),
            EquationContent::Variable("s"),
        ],
        "",
    );
    print_assignments(&sols, lhs_assignment);

    let lhs_assignment = "cdababe";
    let sols = partial_assignments_for_equation(
        &lhs_assignment,
        &[
            EquationContent::Variable("r"),
            EquationContent::Constant("a"),
            EquationContent::Variable("s"),
            EquationContent::Variable("t"),
            EquationContent::Constant("b"),
            EquationContent::Variable("u"),
        ],
        "",
    );
    print_assignments(&sols, lhs_assignment);
}

fn partition_string<'a>(string: &CharOperator<'a>, num_vars: usize) -> Vec<Vec<&'a str>> {
    let partition_positions = (0..string.len() + 1).combinations_with_replacement(num_vars - 1);
    let mut assignments = Vec::with_capacity(partition_positions.try_len().unwrap());
    for partition_position in partition_positions {
        let mut partition_position = partition_position;
        partition_position.insert(0, 0);
        partition_position.push(string.len());
        let values = if string.len() == 0 {
            vec![""; num_vars]
        } else {
            string.multi_substring(&partition_position)
        };
        assignments.push(values);
    }
    assignments
}
