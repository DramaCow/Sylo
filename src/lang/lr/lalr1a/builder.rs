
use std::collections::{HashSet, HashMap, hash_map::Keys};
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

pub struct Reads<'a> {
    symbols: Keys<'a, Symbol, usize>,
    state: usize,
    nullable_ref: &'a [bool], 
    transition_map_ref: &'a HashMap<NonterminalTransition, usize>, 
}

impl Iterator for Reads<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(&symbol) = self.symbols.next() {
            if let Symbol::Variable(B) = symbol {
                if self.nullable_ref[B] {
                    // (p, A) reads (q, B)
                    let transition = NonterminalTransition { state: self.state, var: B };
                    let j = self.transition_map_ref[&transition];
                    return Some(j);
                }
            }
        }
        None
    }
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

        if transitive_closure(&mut read, self.reads_relation(), extend) {
            // cycle detected, TODO: handle errors
        }

        read
    }

    #[must_use]
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

impl<'a> LALR1ABuilder<'a> {
    pub fn reads_relation(&'a self) -> impl FnMut(usize) -> Reads<'a>
    {
        let states = self.lr0a.states();

        move |i: usize| {
            let NonterminalTransition { state: p, var: A } = self.nonterminal_transitions[i];
            let q = states[p].next[&Symbol::Variable(A)];

            Reads {
                symbols: states[q].next.keys(),
                state: q,
                nullable_ref: &self.nullable,
                transition_map_ref: &self.nonterminal_transition_map,
            }
        }
    }
}

// =================
// === INTERNALS === 
// =================

fn extend(a: &mut HashSet<usize>, b: &HashSet<usize>) {
    a.extend(b);
}