
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NonterminalTransition {
    state: usize,
    var: usize,
}

impl<'a> LALR1ABuilder<'a> {
    #[must_use]
    pub fn new(grammar: &'a Grammar) -> Self {
        let lr0a = LR0ABuilder::new(grammar).build();

        let mut nonterminal_transitions: Vec<NonterminalTransition> = Vec::new();
        let mut nonterminal_transition_map: HashMap<NonterminalTransition, usize> = HashMap::new();

        for (p, state) in lr0a.states().iter().enumerate() {
            for &symbol in state.next.keys() {
                if let Symbol::Variable(A) = symbol {
                    let transition = NonterminalTransition { state: p, var: A };
                    let index = nonterminal_transitions.len();
                    nonterminal_transitions.push(transition);
                    nonterminal_transition_map.insert(transition, index);
                }
            }
        }

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

    pub fn read(&self) -> Vec<HashSet<usize>> {
        let states = self.lr0a.states();
        let mut read = self.direct_read();

        let reads = |i: usize| {
            let NonterminalTransition { state: p, var: A } = self.nonterminal_transitions[i];
            let q = states[p].next[&Symbol::Variable(A)];

            let nullable_ref       = &self.nullable;
            let transition_map_ref = &self.nonterminal_transition_map;
            
            states[q].next.keys().filter_map(move |&symbol| {
                if let Symbol::Variable(B) = symbol {
                    if nullable_ref[B] {
                        // (p, A) reads (q, B)
                        let transition = NonterminalTransition { state: q, var: B };
                        let j = transition_map_ref[&transition];
                        return Some(j);
                    }
                }
                None
            })
        };

        if transitive_closure(&mut read, reads, extend) {
            // cycle detected, TODO: handle errors
        }

        read
    }

    pub fn follow(&self) -> Vec<HashSet<usize>> {
        let states = self.lr0a.states();
        let mut follow = self.read();

        let includes_table: Vec<HashSet<usize>> = self.nonterminal_transitions.iter().map(|&transition| {
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
        }).collect();

        let includes = |i: usize| {
            includes_table[i].iter().copied()
        };
        
        if transitive_closure(&mut follow, includes, extend) {
            // cycle detected, TODO: handle errors
        }

        follow
    }
}

// =================
// === INTERNALS === 
// =================

fn extend(a: &mut HashSet<usize>, b: &HashSet<usize>) {
    a.extend(b);
}