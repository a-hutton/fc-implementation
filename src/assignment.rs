use crate::formula::{EquationContent, UNIVERSE_CONSTANT};
use itertools::Itertools;
use std::hash::{Hash, Hasher};

/// Represents an assignment (σ in the literature). Maps variable names to values from the universe
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Assignment<'a> {
    pub keys: Vec<&'a str>,
    pub values: Vec<&'a str>,
    pub universe_constant: &'a str,
}
impl<'a> Assignment<'a> {
    pub fn new(hint: usize) -> Self {
        Assignment {
            keys: Vec::with_capacity(hint),
            values: Vec::with_capacity(hint),
            universe_constant: "",
        }
    }

    pub fn from_vars(vars: &[&'a str], w: &'a str) -> Self {
        let len = vars.len();
        let mut keys = Vec::with_capacity(vars.len());
        for var in vars {
            if !keys.contains(var) {
                keys.push(var);
            }
        }

        Assignment {
            keys,
            values: Vec::with_capacity(len),
            universe_constant: w,
        }
    }

    pub fn join(a: &Assignment<'a>, b: &Assignment<'a>) -> Result<Assignment<'a>, String> {
        let mut new_keys = Vec::with_capacity(a.keys.len() + b.keys.len());
        new_keys.extend(a.keys.clone());
        let mut new_values = Vec::with_capacity(a.values.len() + b.values.len());
        new_values.extend(a.values.clone());

        for (i, var) in b.keys.iter().enumerate() {
            if a.keys.contains(var) {
                let a_val = a.value(var);
                let b_val = b.value(var);
                if a_val != b_val {
                    return Err(format!(
                        "Assignments both contain variable {}, but assign different values. {} != {}",
                        var, a_val, b_val
                    ));
                }
            } else {
                new_keys.push(var);
                new_values.push(b.values[i]);
            }
        }
        Ok(Assignment {
            keys: new_keys,
            values: new_values,
            universe_constant: a.universe_constant,
        })
    }

    pub fn value(&self, var: &str) -> &str {
        if var == UNIVERSE_CONSTANT {
            self.universe_constant
        } else {
            let mut var_idx = 0;
            for key in &self.keys {
                if var == *key {
                    break;
                }
                var_idx += 1;
            }
            self.values[var_idx]
        }
    }

    pub fn insert(&mut self, var: &'a str, value: &'a str) {
        for (i, &key) in self.keys.iter().enumerate() {
            if key == var {
                if self.values.len() <= i {
                    self.values.push(value);
                } else {
                    self.values[i] = value;
                }
                return;
            }
        }
        // if key not found, insert
        self.keys.push(var);
        self.values.push(value);
    }

    pub fn apply(&self, terms: &[EquationContent<'a>]) -> Vec<&'a str> {
        let mut new_terms = Vec::with_capacity(terms.len());
        for term in terms {
            match term {
                EquationContent::Variable(v) => {
                    let val = if *v == UNIVERSE_CONSTANT {
                        self.universe_constant
                    } else {
                        let mut var_idx = 0;
                        for key in &self.keys {
                            if *v == *key {
                                break;
                            }
                            var_idx += 1;
                        }
                        self.values[var_idx]
                    };
                    new_terms.push(val);
                }
                EquationContent::Constant(c) => {
                    new_terms.push(*c);
                }
                EquationContent::UniverseConstant => {
                    new_terms.push(self.universe_constant);
                }
            }
        }
        new_terms
    }

    pub fn extend_with_universe(
        assignment: &Assignment<'a>,
        vars: &[&'a str],
        universe: &[&'a str],
    ) -> Vec<Assignment<'a>> {
        let values = std::iter::repeat_n(universe, vars.len()).multi_cartesian_product();
        let mut assignments = Vec::with_capacity(universe.len() * vars.len());
        for assignment_values in values {
            let mut new_assignment = assignment.clone();
            for i in 0..vars.len() {
                new_assignment.insert(vars[i], assignment_values[i]);
            }
            assignments.push(new_assignment);
        }
        assignments
    }
}

impl Hash for Assignment<'_> {
    /// Hash will return the same values regardless of the internal sorting of the variables
    fn hash<H: Hasher>(&self, state: &mut H) {
        // List of indices sorted by key
        let mut sorted_indices = (0..self.keys.len()).collect_vec();
        sorted_indices.sort_by_key(|&i| self.keys[i]);
        let mut sorted_keys = Vec::with_capacity(sorted_indices.len());
        let mut sorted_values = Vec::with_capacity(sorted_indices.len());
        for index in sorted_indices {
            sorted_keys.push(self.keys[index]);
            sorted_values.push(self.values[index]);
        }

        sorted_keys.hash(state);
        sorted_values.hash(state);
    }
}

#[test]
fn test_hash_equality() {
    fn hash_wrapper<H: Hash>(h: H) -> u64 {
        let mut hasher = std::hash::DefaultHasher::new();
        h.hash(&mut hasher);
        hasher.finish()
    }

    let a1 = Assignment {
        keys: vec!["z", "x", "y"],
        values: vec!["ghi", "abc", "def"],
        universe_constant: "abcdefghi",
    };
    let a2 = Assignment {
        keys: vec!["x", "y", "z"],
        values: vec!["abc", "def", "ghi"],
        universe_constant: "abcdefghi",
    };
    assert_eq!(hash_wrapper(a1), hash_wrapper(a2));
}
