#![allow(non_snake_case)]

use crate::lang::cfg::{Grammar, Symbol};
use crate::lang::lr;
use super::{Item, LR1A};

#[derive(Debug)]
pub struct UncompressedTable {
    actions:    Vec<lr::Action>,    /// lookup what action to perform given state and word
    gotos:      Vec<Option<usize>>, /// lookup what state should be transitioned to after reduction
    reductions: Vec<lr::Reduction>, // alt --> rule and number of symbols
    word_count: usize,
    var_count:  usize,
}

#[derive(Debug)]
pub struct ConstructionError {
    state: usize,
    item: Item,
    action1: lr::Action,
    action2: lr::Action,
}

impl UncompressedTable {
    /// # Errors
    pub fn new(grammar: &Grammar) -> Result<Self, ConstructionError> {
        let lr1a = LR1A::from(grammar);
        
        // let num_words  = word_count + 1; // +1 for eof
        let num_words  = grammar.max_word().map_or(0, |word| word + 1) + 1; // +1 for eof
        let num_vars   = grammar.rule_count() - 1; // implicit start variable not needed in goto table
        let num_states = lr1a.states().len();
        
        let mut actions: Vec<lr::Action>  = vec![lr::Action::Invalid; num_words * num_states];
        let mut gotos: Vec<Option<usize>> = vec![None; num_vars * num_states];

        for (i, state) in lr1a.states().iter().enumerate() {
            for item in &state.items {
                if !item.is_complete(grammar) {
                    let symbol = &item.symbol_at_dot(grammar).unwrap();
                    let index = match symbol {
                        Symbol::Terminal(a) => i * num_words + a + 1,
                        Symbol::Variable(_) => continue,
                    };
                    let action = actions.get_mut(index).unwrap();

                    // check for shift-reduce conflict
                    if let lr::Action::Reduce(_) = action {
                        return Err(ConstructionError { 
                            state: i,
                            item: *item,
                            action1: *action,
                            action2: lr::Action::Shift(state.next[symbol]),
                        });
                    } else {
                        *action = lr::Action::Shift(state.next[symbol]);
                    }
                } else if item.rule < num_vars || item.successor.is_some() { // TODO: second check not necessary?
                    let index = i * num_words + item.successor.map_or(0, |a| a + 1);
                    let action = actions.get_mut(index).unwrap();

                    // check for any conflict
                    if let lr::Action::Invalid = action {
                        *action = lr::Action::Reduce(item.alt);
                    } else {
                        return Err(ConstructionError { 
                            state: i,
                            item: *item,
                            action1: *action,
                            action2: lr::Action::Reduce(item.alt),
                        });
                    }
                } else {
                    actions[i * num_words] = lr::Action::Accept;
                }
            }

            for (var, A) in (0..num_vars).map(|A| (Symbol::Variable(A), A)) {
                gotos[i * num_vars + A] = state.next.get(&var).cloned();
            }
        }

        let reductions = grammar.rules().enumerate().flat_map(|(i, rule)| {
            rule.alts().map(|alt| lr::Reduction { var: i, count: alt.len() }).collect::<Vec<_>>()
        }).collect();

        Ok(Self {
            actions,
            gotos,
            reductions,
            word_count: num_words - 1, // subtract the eof symbol
            var_count: num_vars,
        })
    }
}

impl lr::ParsingTable for UncompressedTable {
    fn action(&self, state: usize, word: Option<usize>) -> lr::Action {
        word.map_or_else(
            || self.actions[state * (self.word_count + 1)],
            |a| self.actions[state * (self.word_count + 1) + a + 1]
        )
    }

    fn goto(&self, state: usize, var: usize) -> Option<usize> {
        self.gotos[state * self.var_count + var]
    }

    fn reduction(&self, alt: usize) -> lr::Reduction {
        self.reductions[alt]
    }
}