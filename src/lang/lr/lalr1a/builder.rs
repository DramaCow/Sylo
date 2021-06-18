#![allow(non_snake_case)]

use std::collections::{HashSet, HashMap};
use crate::lang::cfg::{Grammar, Symbol, nullability};
use crate::lang::lr::{LR0A, LR0ABuilder};
use crate::utils::transitive_closure;

pub struct LALR1ABuilder<'a> {
    grammar: &'a Grammar,
    nullable: Vec<bool>,
    lr0a: LR0A,
    nonterminal_transitions: Vec<NonterminalTransition>,
    nonterminal_transition_map: HashMap<NonterminalTransition, usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NonterminalTransition {
    pub state: usize,
    pub var: usize,
}

impl<'a> LALR1ABuilder<'a> {
    #[must_use]
    pub fn new(grammar: &'a Grammar) -> Self {
        let lr0a = LR0ABuilder::new(grammar).build();

        let nonterminal_transitions: Vec<_> = lr0a.states().iter().enumerate().flat_map(|(p, state)| {
            state.next.keys().filter_map(move |&symbol| {
                if let Symbol::Variable(A) = symbol {
                    Some(NonterminalTransition { state: p, var: A })
                } else {
                    None
                }
            })
        }).collect();

        let nonterminal_transition_map = nonterminal_transitions.iter().enumerate()
            .map(|(i, &transition)| (transition, i)).collect();

        Self {
            grammar,
            nullable: nullability(grammar),
            lr0a,
            nonterminal_transitions,
            nonterminal_transition_map,
        }
    }
}

impl LALR1ABuilder<'_> {
    #[must_use]
    pub fn nonterminal_transitions(&self) -> &[NonterminalTransition] {
        &self.nonterminal_transitions
    }

    #[must_use]
    pub fn direct_read(&self) -> Vec<HashSet<usize>> {
        let states = self.lr0a.states();
        let ntt_count = self.nonterminal_transitions.len();
        let mut direct_read: Vec<HashSet<usize>> = vec![HashSet::new(); ntt_count];

        for (i, &transition) in self.nonterminal_transitions.iter().enumerate() {
            let NonterminalTransition { state: p, var: A } = transition;
            let q = states[p].next[&Symbol::Variable(A)];

            for &symbol in states[q].next.keys() {
                if let Symbol::Terminal(t) = symbol {
                    // (p, A) directly-reads t
                    direct_read[i].insert(t);
                }
            }
        }

        direct_read
    }

    #[must_use]
    pub fn read(&self) -> Vec<HashSet<usize>> {
        let mut read = self.direct_read();
        let reads = self.reads();

        if transitive_closure(&mut read, |i| reads[i].iter().copied(), extend) {
            // cycle detected, TODO: handle errors
        }

        read
    }

    #[must_use]
    pub fn follow(&self) -> Vec<HashSet<usize>> {
        let mut follow = self.read();
        let includes = self.includes();

        if transitive_closure(&mut follow, |i| includes[i].iter().copied(), extend) {
            // cycle detected, TODO: handle errors
        }

        follow
    }

    #[must_use]
    pub fn lookahead(&self) -> Vec<HashSet<usize>> {
        let inconsistent_state_reduction_pairs: Vec<(usize, usize)> = self.lr0a.states().iter()
            .enumerate()
            .filter_map(|(q, state)| {
                if state.items.len() > 1 {
                    Some(state.items.iter().filter_map(move |item| {
                        if item.is_complete(self.grammar) {
                            Some((q, item.alt))
                        } else {
                            None
                        }
                    }))
                } else {
                    None
                }
            }).flatten().collect();

        println!("{:?}", inconsistent_state_reduction_pairs);
        
        todo!()
    }

    #[must_use]
    pub fn reads(&self) -> Vec<HashSet<usize>> {
        // NOTE: this doesn't need to be stored: can be computed on the fly.

        let states = self.lr0a.states();

        self.nonterminal_transitions.iter().map(|&transition| {
            let NonterminalTransition { state: p, var: A } = transition;
            let q = states[p].next[&Symbol::Variable(A)];

            states[q].next.keys().filter_map(|&symbol| {
                if let Symbol::Variable(B) = symbol {
                    if self.nullable[B] {
                        // (p, A) reads (q, B)
                        let transition = NonterminalTransition { state: q, var: B };
                        let j = self.nonterminal_transition_map[&transition];
                        return Some(j);
                    }
                }
                None
            }).collect()
        }).collect()
    }

    #[must_use]
    pub fn includes(&self) -> Vec<HashSet<usize>> {
        let states = self.lr0a.states();

        self.nonterminal_transitions.iter().map(|&transition| {
            let NonterminalTransition { state: p, var: B } = transition;
            let mut successors = HashSet::new();

            for alt in self.grammar.rule(B).alts() {
                let mut q = p;

                for (i, &symbol) in alt.iter().enumerate() {
                    if let Symbol::Variable(A) = symbol {
                        let nullable_gamma = alt[i+1..].iter().all(|&symbol| match symbol {
                            Symbol::Terminal(_) => false,
                            Symbol::Variable(C) => self.nullable[C],
                        });

                        if nullable_gamma {
                            successors.insert(self.nonterminal_transition_map[&NonterminalTransition { state: q, var: A }]);
                        }
                    }

                    q = states[q].next[&symbol];
                }
            }

            successors
        }).collect()
    }
}

// pub struct Reads<'a> {
//     state: usize,
//     symbols: Keys<'a, Symbol, usize>,
//     nullable_ref: &'a [bool], 
//     transition_map_ref: &'a HashMap<NonterminalTransition, usize>, 
// }

// impl Iterator for Reads<'_> {
//     type Item = usize;

//     fn next(&mut self) -> Option<Self::Item> {
//         while let Some(&symbol) = self.symbols.next() {
//             if let Symbol::Variable(B) = symbol {
//                 if self.nullable_ref[B] {
//                     // (p, A) reads (q, B)
//                     let transition = NonterminalTransition { state: self.state, var: B };
//                     let j = self.transition_map_ref[&transition];
//                     return Some(j);
//                 }
//             }
//         }
//         None
//     }
// }

// =================
// === INTERNALS === 
// =================

fn extend(a: &mut HashSet<usize>, b: &HashSet<usize>) {
    a.extend(b);
}