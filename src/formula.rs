use crate::{generate_factors, print_solutions, strutils, Substitution};
use itertools::Itertools;
use std::collections::HashSet;
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

/// A common interface for all word formula types
pub trait Formula: fmt::Display + fmt::Debug {
    fn free_vars(&self) -> Vec<&str>;
    fn check_substitution(&self, substitution: &Substitution, universe: &[&str]) -> bool;
    fn all_solutions<'b>(&'b self, universe: &'b [&'b str]) -> Vec<Substitution<'b>>;
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
    fn check_substitution(&self, substitution: &Substitution, _: &[&str]) -> bool {
        // TODO?
        // if !substitution.contains_key(UNIVERSE_CONSTANT) {
        //     panic!("Missing universe constant `$U` (𝔲) in substitution")
        // }
        let lhs_vec = vec![EquationContent::Variable(self.lhs_variable)];
        let lhs_sub = substitution.apply(&lhs_vec).join("");
        let rhs_sub = substitution.apply(&self.rhs).join("");

        lhs_sub == rhs_sub
    }

    fn all_solutions<'b>(&'b self, universe: &'b [&'b str]) -> Vec<Substitution<'b>> {
        // TODO - special case for $U
        let mut substitutions = Vec::new();
        for &lhs_value in universe {
            // For each possible value for the lhs variable
            let subs_for_val = self.rhs_assignments(lhs_value);
            substitutions.extend(subs_for_val);
        }
        substitutions
    }
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

impl<'a> AtomicWordEquation<'a> {
    fn rhs_assignments(&self, lhs_value: &'a str) -> Vec<Substitution<'a>> {
        let lhs_chars = strutils::CharOperator::new(lhs_value);

        // TODO - filter out duplicates.... somehow
        // TODO - filter out obvious fails as consts must be in sequence
        let mut pattern_positions = Vec::with_capacity(self.rhs.len());
        for p in self.rhs.iter() {
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
        for i in 0..self.rhs.len() {
            let is_start = i == 0;
            let is_final = i == self.rhs.len() - 1;
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
            let pattern_element = &self.rhs[i];

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

            let mut sub = Substitution::new();
            // TODO?
            // sub.insert(UNIVERSE_CONSTANT, "");
            sub.insert(self.lhs_variable, lhs_value);
            for (i, pattern_element) in self.rhs.iter().enumerate() {
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
            if self.check_substitution(&sub, &[]) {
                substitutions.push(sub);
            }
        }

        substitutions
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

#[test]
fn test_new_method() {
    let eq = AtomicWordEquation::new(
        "x",
        vec![
            EquationContent::Variable("y"),
            EquationContent::Constant("a"),
            EquationContent::Variable("z"),
            EquationContent::Constant("b"),
            EquationContent::Variable("q"),
        ],
    );
    let sols = eq.rhs_assignments("bbabab");
    print_solutions(&sols, "bbabab");
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

    fn all_solutions<'b>(&'b self, universe: &'b [&'b str]) -> Vec<Substitution<'b>> {
        let mut lhs_solutions = self.lhs.all_solutions(universe);
        let mut i = 0;
        while i < lhs_solutions.len() {
            if !self.rhs.check_substitution(&lhs_solutions[i], universe) {
                lhs_solutions.swap_remove(i);
            }
            i += 1;
        }
        lhs_solutions
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

    fn all_solutions(&'_ self, universe: &[&str]) -> Vec<Substitution> {
        todo!()
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
    pub fn new<'a>(inner: Box<dyn Formula + 'a>) -> NegativeFormula<'a> {
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

    fn all_solutions(&'_ self, universe: &[&str]) -> Vec<Substitution<'_>> {
        todo!()
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

    /// Checks a [`Substitution`] by calling [`generate_factors`] on the universe constant given
    /// in the substitution, and for each possible value to assign to the bound variable _x_, a
    /// new substitution is checked on the inner [`Formula::check_substitution`], returning
    /// `true` when the first valid substitution is found
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

    fn all_solutions(&'_ self, universe: &[&str]) -> Vec<Substitution> {
        todo!()
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

    /// Checks a [`Substitution`] by calling [`generate_factors`] on the universe constant given
    /// in the substitution, and for each possible value to assign to the bound variable _x_, a
    /// new substitution is checked on the inner [`Formula::check_substitution`], returning
    /// `true` if every new substitution holds
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

    fn all_solutions(&'_ self, universe: &[&str]) -> Vec<Substitution> {
        todo!()
    }
}

impl fmt::Display for UniversalFormula<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let _ = write!(f, "∀{x}: (", x = self.bound_var);
        let _ = self.inner.fmt(f);
        write!(f, ")")
    }
}
