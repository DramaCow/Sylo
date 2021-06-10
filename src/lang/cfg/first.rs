use std::collections::BTreeSet;
use std::iter::once;
use super::{Grammar, Symbol};
use std::ops::Index;
use crate::utils::transitive_closure;

/// For non-terminal A, first[A] is the set of terminals that can appear at
/// the start of a sentence derived from A. This does not include epsilons;
/// for this behaviour, see [nullability]().
#[derive(Debug)]
pub struct First {
    firsts: Vec<usize>,
    var_ranges: Vec<usize>,
}

impl First {
    #[must_use]
    pub fn new(grammar: &Grammar, nullable: &[bool]) -> Self {
        let var_firsts = compute_var_firsts_v2(grammar, nullable);
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
    type Output = [usize];

    fn index(&self, var: usize) -> &Self::Output {
        &self.firsts[self.var_ranges[var]..self.var_ranges[var+1]]
    }
}

// =================
// === INTERNALS ===
// =================


/// Let `A ~ B` hold iff there is a production of the form `A -> pX...`, where
/// `p` is a nullable sequence of symbols. Let `~+` be the transitive closure
/// of relation `~`.
/// 
/// ```text
/// first[A] = { t in T | A ~+ t }
///          = { t in T | A ~ t or A ~ B ~+ t }
///          = { t in T | A ~ t } U { t in first[B] | A ~ B }
///          = first'[A] U union({ first[B] | A ~ B })
/// ```
/// 
/// Hence, we can compute `first` by applying the transitive closure algorithm.
fn compute_var_firsts_v2(grammar: &Grammar, nullable: &[bool]) -> Vec<BTreeSet<usize>> {
    let var_count = grammar.var_count();
    let mut first = vec![BTreeSet::new(); var_count];
    let mut dependency_matrix = vec![false; var_count * var_count];
    
    // Initialise first to trivial values and fill dependency_matrix
    for (A, rule) in grammar.rules().enumerate() {
        for alt in rule.alts() {            
            for &symbol in alt {
                match symbol {
                    Symbol::Terminal(a) => {
                        first[A].insert(a);
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

    transitive_closure(&mut first, left_dependencies, extend);

    first
}