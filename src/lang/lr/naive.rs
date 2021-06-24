#![allow(non_snake_case)]

use crate::lang::cfg::{Grammar, Symbol};
use super::{LR1ABuilder, Action, Reduction, LRTable};

#[derive(Debug)]
pub struct NaiveLRTable {
    actions:    Vec<Action>,        /// lookup what action to perform given state and word
    gotos:      Vec<Option<usize>>, /// lookup what state should be transitioned to after reduction
    reductions: Vec<Reduction>,     // production --> rule and number of symbols
    word_count: usize,
    var_count:  usize,
}

#[derive(Debug)]
pub struct ConstructionError {
    state: usize,
    conflict: Conflict,
}

#[derive(Debug)]
pub enum Conflict {
    ShiftReduce { word: usize, next_state: usize, production: usize },
    ReduceReduce { production1: usize, production2: usize },
}

impl NaiveLRTable {
    /// No conflicts allowed.
    /// 
    /// # Errors
    pub fn new(grammar: &Grammar) -> Result<Self, ConstructionError> {
        Self::with_conflict_resolution(grammar, |conflict: Conflict| { Err(conflict) })
    }

    /// # Errors
    pub fn with_conflict_resolution<F>(grammar: &Grammar, mut conflict_resolution: F) -> Result<Self, ConstructionError>
    where
        F: FnMut(Conflict) -> Result<Action, Conflict>,
    {
        let lr1a = LR1ABuilder::new(grammar).build();
        
        let word_count = grammar.word_count() + 1; // +1 for eof
        let var_count  = grammar.var_count() - 1; // implicit start variable not needed in goto table
        let num_states = lr1a.states().len();
        
        let reductions: Vec<_> = grammar.rules().enumerate().flat_map(|(i, rule)| {
            rule.alts().map(move |alt| Reduction { var: i, count: alt.len() })
        }).collect();

        let mut actions: Vec<Action>  = vec![Action::Invalid; word_count * num_states];
        let mut gotos: Vec<Option<usize>> = vec![None; var_count * num_states];

        for (i, state) in lr1a.states().iter().enumerate() {
            for item in &state.items {
                if !item.lr0_item.is_complete(grammar) {
                    let symbol = item.lr0_item.symbol_at_dot(grammar).unwrap();
                    if let Symbol::Terminal(word) = symbol {
                        // CASE 1: item is incomplete and has a terminal symbol at dot.

                        let action = actions.get_mut(i * word_count + word + 1).unwrap();
                        let next_state = state.next[&symbol];
    
                        // Note: shift-shift conflicts cannot occur
                        if let Action::Reduce(production) = *action {
                            *action = conflict_resolution(Conflict::ShiftReduce { word, next_state, production })
                                .map_err(|conflict| ConstructionError { state: i, conflict })?;
                        } else {
                            *action = Action::Shift(next_state);
                        }
                    }
                } else if reductions[item.lr0_item.production].var < var_count || item.lookahead.is_some() { // TODO: second check not necessary?
                    // CASE 2: item is complete and does not have the start symbol on LHS.

                    let column = item.lookahead.map_or(0, |a| a + 1);
                    let action = actions.get_mut(i * word_count + column).unwrap();
                    
                    match *action {
                        Action::Shift(state) => {
                            let word = column - 1;
                            *action = conflict_resolution(Conflict::ShiftReduce { word, next_state: state, production: item.lr0_item.production })
                                .map_err(|conflict| ConstructionError { state: i, conflict })?;
                        }
                        Action::Reduce(production1) => {
                            *action = conflict_resolution(Conflict::ReduceReduce { production1, production2: item.lr0_item.production })
                                .map_err(|conflict| ConstructionError { state: i, conflict })?;
                        }
                        _ => {
                            *action = Action::Reduce(item.lr0_item.production);
                        }
                    }
                } else {
                    // CASE 3: item is complete and has start symbol on LHS.
                    actions[i * word_count] = Action::Accept;
                }
            }

            for (var, A) in (0..var_count).map(|A| (Symbol::Variable(A), A)) {
                gotos[i * var_count + A] = state.next.get(&var).copied();
            }
        }

        Ok(Self {
            actions,
            gotos,
            reductions,
            word_count: word_count - 1, // subtract the eof symbol
            var_count: var_count,
        })
    }
}

impl LRTable for NaiveLRTable {
    fn action(&self, state: usize, word: Option<usize>) -> Action {
        word.map_or_else(
            || self.actions[state * (self.word_count + 1)],
            |a| self.actions[state * (self.word_count + 1) + a + 1]
        )
    }

    fn goto(&self, state: usize, var: usize) -> Option<usize> {
        self.gotos[state * self.var_count + var]
    }

    fn reduction(&self, alt: usize) -> Reduction {
        self.reductions[alt]
    }
}