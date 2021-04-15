#![allow(non_snake_case)]

use crate::lang::{
    cfg::{Grammar, Symbol},
    lr1::{LR1ABuilder},
    lr::{Action, Reduction, ParsingTable, Precedence, Associativity},
};

#[derive(Debug)]
pub struct ArrayParsingTable {
    actions:    Vec<Action>,        /// lookup what action to perform given state and word
    gotos:      Vec<Option<usize>>, /// lookup what state should be transitioned to after reduction
    reductions: Vec<Reduction>,     // alt --> rule and number of symbols
    word_count: usize,
    var_count:  usize,
}

#[derive(Debug)]
pub enum ConstructionError {
    ShiftReduce { word: usize, alt: usize },
    ReduceReduce { alt1: usize, alt2: usize },
}

impl ArrayParsingTable {
    /// # Errors
    pub fn new(grammar: &Grammar, token_precedence: &[Option<Precedence>], production_precedence: &[Option<Precedence>]) -> Result<Self, ConstructionError> {
        let lr1a = LR1ABuilder::new(grammar).build();
        
        let word_count = grammar.max_word().map_or(0, |word| word + 1) + 1; // +1 for eof
        let var_count  = grammar.var_count() - 1; // implicit start variable not needed in goto table
        let num_states = lr1a.states().len();
        
        let reductions: Vec<_> = grammar.rules().enumerate().flat_map(|(i, rule)| {
            rule.alts().map(|alt| Reduction { var: i, count: alt.len() }).collect::<Vec<_>>()
        }).collect();

        let mut actions: Vec<Action>  = vec![Action::Invalid; word_count * num_states];
        let mut gotos: Vec<Option<usize>> = vec![None; var_count * num_states];

        for (i, state) in lr1a.states().iter().enumerate() {
            for item in &state.items {
                if !item.lr0_item.is_complete(grammar) {
                    let symbol = item.lr0_item.symbol_at_dot(grammar).unwrap();
                    let word = match symbol {
                        Symbol::Terminal(a) => a,
                        Symbol::Variable(_) => continue,
                    };
                    let index = i * word_count + word + 1;
                    let action = actions.get_mut(index).unwrap();

                    // check for shift-reduce conflict
                    if let Action::Reduce(alt) = *action {
                        return Err(ConstructionError::ShiftReduce { word, alt });
                    } else {
                        *action = Action::Shift(state.next[&symbol]);
                    }
                } else if reductions[item.lr0_item.alt].var < var_count || item.lookahead.is_some() { // TODO: second check not necessary?
                    let index = i * word_count + item.lookahead.map_or(0, |a| a + 1);
                    let action = actions.get_mut(index).unwrap();
                    
                    // check for any conflict
                    match *action {
                        Action::Reduce(alt1) => {
                            return Err(ConstructionError::ReduceReduce { alt1, alt2: item.lr0_item.alt });
                        }
                        Action::Shift(word) => {
                            return Err(ConstructionError::ShiftReduce { word, alt: item.lr0_item.alt });
                        }
                        _ => {
                            *action = Action::Reduce(item.lr0_item.alt);
                        }
                    }
                } else {
                    actions[i * word_count] = Action::Accept;
                }
            }

            for (var, A) in (0..var_count).map(|A| (Symbol::Variable(A), A)) {
                gotos[i * var_count + A] = state.next.get(&var).cloned();
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

impl ParsingTable for ArrayParsingTable {
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