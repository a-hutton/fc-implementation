use crate::formula::{EquationContent, UNIVERSE_CONSTANT};

/// Represents an assignment (σ in the literature). Maps variable names to values from the universe
#[derive(Eq, PartialEq, Hash, Debug, Clone)]
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
        let mut subs = Vec::with_capacity(universe.len() * vars.len());
        for &var in vars {
            for &val in universe {
                let mut new_sub = assignment.clone();
                new_sub.insert(var, val);
                subs.push(new_sub);
            }
        }
        subs
    }
}
