use std::collections::BTreeSet;
use std::iter::once;
use super::{Grammar, Symbol};
use std::ops::Index;

/// A utility struct that, for each unique variable present in a 
/// grammar, stores the set of terminals (the first set) that can
/// appear at the start of a sentence derived from that symbol.
/// 
/// Sets of tokens are represented as slices of type `Option<usize>`,
/// where `None` represents epsilon.  
/// 
/// NOTE: Trivially, the first set of each terminal symbol is the 
/// set containing only itself, and the first of epsilon is 
/// the set containing None. For this reason, they are omitted from
/// this structure.
#[derive(Debug)]
pub struct First {
    firsts: Vec<Option<usize>>,
    var_ranges: Vec<usize>,
}

impl First {
    #[must_use]
    pub fn new(grammar: &Grammar) -> Self {
        let var_firsts = compute_var_firsts(grammar);
        let var_ranges = once(0)
            .chain(
                var_firsts.iter()
                .map(BTreeSet::len)
                .scan(0, |cumsum, len| { *cumsum += len; Some(*cumsum) }))
            .collect();

        Self {
            firsts: var_firsts.into_iter().flatten().collect(),
            var_ranges,
        }
    }
}

impl Index<usize> for First {
    type Output = [Option<usize>];

    fn index(&self, var: usize) -> &Self::Output {
        &self.firsts[self.var_ranges[var]..self.var_ranges[var+1]]
    }
}

// =================
// === INTERNALS ===
// =================

/// Constructs the first sets for each unique variable in grammar.
fn compute_var_firsts(grammar: &Grammar) -> Vec<BTreeSet<Option<usize>>> {
    let mut first = vec![BTreeSet::<Option<usize>>::new(); grammar.var_count()];

    let mut done = false;
    while !done {
        done = true;

        for (A, rule) in grammar.rules().enumerate() {
            for alt in rule.alts() {
                let mut rhs = BTreeSet::new();

                if alt.is_empty() {
                    rhs.insert(None);
                } else {
                    for (j, &symbol) in alt.iter().enumerate() {
                        match symbol {
                            // If the current grammar symbol being considered
                            // is a terminal, then succeeding grammar symbols in the alt
                            // cannot contribute to the first of the rule.
                            Symbol::Terminal(a) => {
                                rhs.insert(Some(a));
                                break;
                            },
                            Symbol::Variable(A) => {
                                // If a grammar symbol being considered is not the last of an alt 
                                // and its first contains an epsilon, then the succeeding grammar
                                // symbol also contributes to the first of the rule.
                                if first[A].contains(&None) && j < alt.len() - 1 {
                                    rhs.extend(first[A].iter().filter(|a| Option::<usize>::is_some(a)));
                                } else {
                                    rhs.extend(first[A].iter());
                                    break;
                                }
                            },
                        }
                    }
                }

                if !rhs.is_subset(&first[A]) {
                    first[A].extend(rhs);
                    done = false;
                }
            }
        }
    }

    first
}