#![allow(non_snake_case)]

use crate::langcore::cfg::{Grammar, Symbol};
use super::{Action, Reduction, Conflict, NaiveLR1Table, ConstructionError};

pub trait ItemSets {
    type Item;

    fn state_count(&self) -> usize;

    fn items(&self, state: usize) -> &[Self::Item];
    fn transition(&self, state: usize, symbol: Symbol) -> Option<usize>;
    
    fn production(&self, item: &Self::Item) -> usize;
    fn is_complete(&self, item: &Self::Item) -> bool;
    fn symbol_at_dot(&self, item: &Self::Item) -> Option<Symbol>;
}

pub trait Lookaheads<'a>: ItemSets {
    type Output;

    fn lookaheads(&'a self, state: usize, item: &Self::Item) -> Self::Output;
}

pub trait BuildLR1Table<'a> {
    /// # Errors
    fn build_lr1_table<F>(&'a self, grammar: &Grammar, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError>
    where
        F: FnMut(Conflict) -> Result<Action, Conflict>;
}

impl<'a, T> BuildLR1Table<'a> for T
where
    T: Lookaheads<'a>,
    <T as Lookaheads<'a>>::Output: IntoIterator<Item = Option<usize>>,
{
    /// # Errors
    fn build_lr1_table<F>(&'a self, grammar: &Grammar, mut conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError>
    where
        F: FnMut(Conflict) -> Result<Action, Conflict>,
    {       
        let word_count = grammar.word_count() + 1; // +1 for eof
        let var_count = grammar.var_count() - 1; // implicit start variable not needed in goto table
        let num_states = self.state_count();

        let mut table = NaiveLR1Table {
            actions: vec![Action::Invalid; word_count * num_states],
            gotos: vec![None; var_count * num_states],
            reductions:
                grammar.rules().enumerate().flat_map(|(i, rule)| {
                    rule.alts().map(move |alt| Reduction { var: i, count: alt.len() })
                }).collect(),
            word_count,
            var_count,
        };

        for i in 0..num_states {
            for item in self.items(i) {
                if !self.is_complete(item) {
                    let symbol = self.symbol_at_dot(item).unwrap();
                    if let Symbol::Terminal(word) = symbol {
                        // CASE 1: item is incomplete and has a terminal symbol at dot.

                        let action = table.actions.get_mut(i * word_count + word + 1).unwrap();
                        let next_state = self.transition(i, symbol).unwrap();
    
                        // Note: shift-shift conflicts cannot occur
                        if let Action::Reduce(production) = *action {
                            *action = conflict_resolution(Conflict::ShiftReduce { word, next_state, production })
                                .map_err(|conflict| ConstructionError { state: i, conflict })?;
                        } else {
                            *action = Action::Shift(next_state);
                        }
                    }
                } else if table.reductions[self.production(item)].var < var_count {
                    // CASE 2: item is complete and does not have the start symbol on LHS.

                    for lookahead in self.lookaheads(i, item) {
                        let column = lookahead.map_or(0, |a| a + 1);
                        let action = table.actions.get_mut(i * word_count + column).unwrap();
                        
                        match *action {
                            Action::Shift(state) => {
                                *action = conflict_resolution(Conflict::ShiftReduce { word: column - 1, next_state: state, production: self.production(item) })
                                    .map_err(|conflict| ConstructionError { state: i, conflict })?;
                            }
                            Action::Reduce(production1) => {
                                *action = conflict_resolution(Conflict::ReduceReduce { production1, production2: self.production(item) })
                                    .map_err(|conflict| ConstructionError { state: i, conflict })?;
                            }
                            _ => {
                                *action = Action::Reduce(self.production(item));
                            }
                        }
                    }
                } else {
                    // CASE 3: item is complete and has start symbol on LHS (lookahead will always be eof).
                    table.actions[i * word_count] = Action::Accept;
                }
            }

            for (var, A) in (0..var_count).map(|A| (Symbol::Variable(A), A)) {
                table.gotos[i * var_count + A] = self.transition(i, var);
            }
        }

        Ok(table)
    } 
}