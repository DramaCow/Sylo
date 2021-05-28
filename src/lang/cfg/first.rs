use std::collections::BTreeSet;
use std::iter::once;
use super::{Grammar, Symbol};
use std::ops::Index;

use crate::utils::transitive_closure;

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
fn compute_var_firsts_v2(grammar: &Grammar, nullable: &[bool]) -> Vec<BTreeSet<usize>> {
    let var_count = grammar.var_count();
    let mut trivial_first = vec![BTreeSet::new(); var_count];
    let mut dependency_matrix = vec![false; var_count * var_count];
    
    // Initialise first to trivial values and fill left_dependent matrix
    for (A, rule) in grammar.rules().enumerate() {
        for alt in rule.alts() {            
            for &symbol in alt {
                match symbol {
                    Symbol::Terminal(a) => {
                        trivial_first[A].insert(a);
                        break;
                    }
                    Symbol::Variable(B) => {
                        dependency_matrix[A * var_count + B] = true;
                        if !nullable[B] {
                            break;
                        }
                    }
                };
            }
        }
    }

    let dependency_matrix_ref = &dependency_matrix;
    let left_dependencies = |A: usize| {
        (0..var_count).filter(move |B| dependency_matrix_ref[A * var_count + B])
    };

    let extend = |A: &mut BTreeSet<usize>, B: &BTreeSet<usize>| {
        A.extend(B);
    };

    transitive_closure(trivial_first, left_dependencies, extend)
}

/// Constructs the first sets for each unique variable in grammar.
fn compute_var_firsts(grammar: &Grammar) -> Vec<BTreeSet<Option<usize>>> {
    let mut first = vec![BTreeSet::new(); grammar.var_count()];

    let mut done = false;
    while !done {
        done = true;

        for (A, rule) in grammar.rules().enumerate() {
            for alt in rule.alts() {
                let mut rhs = BTreeSet::new();

                if alt.is_empty() {
                    // alt is epsilon
                    rhs.insert(None);
                } else {
                    for (j, &symbol) in alt.iter().enumerate() {
                        match symbol {
                            // If the current grammar symbol being considered
                            // is a terminal, then succeeding grammar symbols
                            // in the alt cannot contribute to first(A).
                            Symbol::Terminal(b) => {
                                rhs.insert(Some(b));
                                break;
                            },
                            Symbol::Variable(B) => {
                                // If B is not the last symbol in the rhs of the production
                                // and B is nullable (first(B) contains epsilon), then the
                                // succeeding grammar symbol also contributes to first(A).
                                if j < alt.len() - 1 && first[B].contains(&None) {
                                    rhs.extend(first[B].iter().filter(|b: &&Option<usize>| b.is_some()));
                                } else {
                                    rhs.extend(first[B].iter());
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