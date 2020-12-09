use std::iter::once;
use std::collections::BTreeSet;
use super::{Grammar, Symbol};

/// A utility struct that, for each unique symbol present in a 
/// grammar, stores the set of terminals (the first set) that can
/// appear at the start of a sentence derived from that symbol.
/// 
/// Trivially, the first set of each terminal symbol is the 
/// set containing only itself, and the first of epsilon is 
/// the set containing None. 
#[derive(Debug)]
pub struct First {
    firsts: Vec<Option<usize>>,
    var_ranges: Vec<usize>,
}

impl First {
    #[must_use]
    pub fn new(grammar: &Grammar) -> Self {
        let var_firsts = compute_var_firsts(&grammar);
        let var_offset = grammar.termcount + 1;
        let var_ranges = once(var_offset)
            .chain(var_firsts.iter()
                .map(BTreeSet::len)
                .scan(var_offset, |cumsum, len| { *cumsum += len; Some(*cumsum) }))
            .collect();
        let firsts = once(None)
            .chain((0..var_offset-1).map(Some))
            .chain(var_firsts.into_iter().flatten())
            .collect();

        Self {
            firsts,
            var_ranges,
        }
    }

    #[must_use]
    pub fn get(&self, symbol: &Option<Symbol>) -> &[Option<usize>] {
        match symbol {
            None                      => &self.firsts[0..=0],
            Some(Symbol::Terminal(a)) => &self.firsts[1+a..=1+a],
            Some(Symbol::Variable(A)) => &self.firsts[self.var_ranges[*A]..self.var_ranges[A+1]],
        }
    }

    #[must_use]
    pub fn get_union(&self, symbol: &Option<Symbol>, successor: &Option<Symbol>) -> FirstUnion {
        let first = self.get(&symbol);

        // if first contains epsilon, it is guaranteed to be at index 0
        if first[0].is_none() {
            FirstUnion::new(&first[1..], self.get(&successor))
        } else {
            FirstUnion::new(first, &[])
        }
    }
}

/// Leverages the fact first sets are stored as sorted vectors in First
/// in order to allow quick iteration over the union.
pub struct FirstUnion<'a> {
    first1: &'a [Option<usize>],
    first2: &'a [Option<usize>],
    i1: usize,
    i2: usize,
}

impl<'a> FirstUnion<'a> {
    #[must_use]
    fn new(first1: &'a [Option<usize>], first2: &'a [Option<usize>]) -> Self {
        Self {
            first1,
            first2,
            i1: 0,
            i2: 0,
        }
    }
}

impl<'a> Iterator for FirstUnion<'a> {
    type Item = &'a Option<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let (i1, i2) = (self.i1, self.i2);

        if i1 < self.first1.len() {
            if i2 < self.first2.len() {
                match (&self.first1[i1], &self.first2[i2]) {
                    (a, b) if a < b => {
                        self.i1 += 1;
                        Some(a)
                    }
                    (a, b) if a > b => {
                        self.i2 += 1;
                        Some(b)
                    }
                    (a, _) => {
                        self.i1 += 1;
                        self.i2 += 1;
                        Some(a)
                    }
                }
            } else {
                self.i1 += 1;
                Some(&self.first1[i1])
            }
        } else if self.i2 < self.first2.len() {
            self.i2 += 1;
            Some(&self.first2[i2])
        } else {
            None
        }
    }
}

// =================
// === INTERNALS ===
// =================

/// Constructs the first sets for each unique variable in grammar.
fn compute_var_firsts(grammar: &Grammar) -> Vec<BTreeSet<Option<usize>>> {
    let num_vars = grammar.rule_count();
    let mut first = vec![BTreeSet::<Option<usize>>::new(); num_vars];

    let mut done = false;
    while !done {
        done = true;

        for (A, rule) in grammar.rules().enumerate() {
            for alt in rule.alts() {
                let mut rhs = BTreeSet::new();

                if alt.is_empty() {
                    rhs.insert(None);
                } else {
                    for (j, symbol) in alt.iter().enumerate() {
                        match symbol {
                            // If the current grammar symbol being considered
                            // is a terminal, then succeeding grammar symbols in the alt
                            // cannot contribute to the first of the rule.
                            Symbol::Terminal(a) => {
                                rhs.insert(Some(*a));
                                break;
                            },
                            Symbol::Variable(A) => {
                                // If a grammar symbol being considered is not the last of an alt 
                                // and its first contains an epsilon, then the succeeding grammar
                                // symbol also contributes to the first of the rule.
                                if first[*A].contains(&None) && j < alt.len() - 1 {
                                    rhs.extend(first[*A].iter().filter(|a| Option::<usize>::is_some(a)));
                                } else {
                                    rhs.extend(first[*A].iter());
                                    break;
                                }
                            },
                        }
                    }
                }

                if !rhs.is_subset(&first[A]) {
                    first.get_mut(A).unwrap().extend(rhs);
                    done = false;
                }
            }
        }
    }

    first
}