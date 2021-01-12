#![allow(non_snake_case)]

use crate::lang::cfg::{Grammar, Symbol, lr1};

pub use self::parse::{Instruction, Parse, ParseError};

#[derive(Debug, Clone)]
pub enum Command {
    Emit,
    Skip,
}

pub struct SynAnalyzerDef {
    pub labels:   Vec<String>,
    pub grammar:  Grammar,
}

#[derive(Debug)]
pub struct SynAnalyzer {
    labels:       Vec<String>,
    actions:      Vec<Action>,
    gotos:        Vec<Option<usize>>,
    reductions:   Vec<Reduction>,     // alt --> rule + number of symbols
    term_count:   usize,
}

#[derive(Debug)]
pub enum CompileError {
    Conflict { state: usize, item: lr1::Item, },//action1: Action, action2: Action },
}

impl SynAnalyzer {
    /// # Errors
    pub fn try_compile(def: &SynAnalyzerDef) -> Result<Self, CompileError> {
        let dfa = lr1::DFA::from(&def.grammar);
        
        let num_vars = def.grammar.rule_count() - 1; // implicit start variable not needed in goto table
        let num_words = def.grammar.term_count + 1; // +1 for eof
        let num_states = dfa.states.len();
        
        let mut actions: Vec<Action>      = vec![Action::Invalid; num_words * num_states];
        let mut gotos: Vec<Option<usize>> = vec![None; num_vars * num_states];

        for (i, state) in dfa.states.iter().enumerate() {
            for item in &state.items {
                if !def.grammar.is_complete(item) {
                    let symbol = &def.grammar.symbol_at_dot(item).unwrap();
                    let index = match symbol {
                        Symbol::Terminal(a) => i * num_words + a + 1,
                        Symbol::Variable(_) => continue,
                    };
                    let action = actions.get_mut(index).unwrap();

                    // check for shift-reduce conflict
                    if let Action::Reduce(_) = action {
                        return Err(CompileError::Conflict { 
                            state: i,
                            item: *item,
                            // action1: *action,
                            // action2: Action::Shift(state.next[symbol]),
                        });
                    } else {
                        *action = Action::Shift(state.next[symbol]);
                    }
                } else if item.rule != num_vars || item.successor.is_some() { // TODO: second check not necessary?
                    let index = i * num_words + item.successor.map_or(0, |a| a + 1);
                    let action = actions.get_mut(index).unwrap();

                    // check for any conflict
                    if let Action::Invalid = action {
                        *action = Action::Reduce(item.alt);
                    } else {
                        return Err(CompileError::Conflict { 
                            state: i,
                            item: *item,
                            // action1: *action,
                            // action2: Action::Reduce(item.alt),
                        });
                    }
                } else {
                    actions[i * num_words] = Action::Accept;
                }
            }

            for (var, A) in (0..num_vars).map(|A| (Symbol::Variable(A), A)) {
                gotos[i * num_vars + A] = state.next.get(&var).cloned();
            }
        }

        let reductions = def.grammar.rules().enumerate().flat_map(|(i, rule)| {
            rule.alts().map(|alt| Reduction{ var: i, count: alt.len() }).collect::<Vec<_>>()
        }).collect();

        Ok(Self {
            labels: def.labels.to_vec(),
            actions,
            gotos,
            reductions,
            term_count: def.grammar.term_count,
        })
    }

    #[must_use]
    pub fn parse<T: IntoIterator<Item=usize>>(&self, words: T) -> Parse<T::IntoIter> {
        Parse::new(&self, words.into_iter())
    }
}

// =================
// === INTERNALS ===
// =================

mod parse;

impl SynAnalyzer {  
    fn var_count(&self) -> usize {
        self.labels.len()
    }
}

#[derive(Debug, Clone, Copy)]
struct Reduction {
    var: usize,
    count: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Action {
    Invalid,
    Accept,
    Shift(usize),
    Reduce(usize),
}

impl SynAnalyzer {
    fn action(&self, state: usize, word: Option<usize>) -> Action {
        word.map_or(
            self.actions[state * (self.term_count + 1)],
            |a| self.actions[state * (self.term_count + 1) + a + 1]
        )
    }

    fn goto(&self, state: usize, var: usize) -> Option<usize> {
        self.gotos[state * self.var_count() + var]
    }
}

#[cfg(test)]
mod tests;