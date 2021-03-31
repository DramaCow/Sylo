#![allow(non_snake_case)]

use crate::lang::{
    cfg::{Grammar, Symbol},
    lr1::{LR1Item, LR1ABuilder},
    lr::{Action, Reduction, ParsingTable},
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
pub struct ConstructionError {
    state: usize,
    item: LR1Item,
    action1: Action,
    action2: Action,
}

impl ArrayParsingTable {
    /// # Errors
    pub fn new(grammar: &Grammar) -> Result<Self, ConstructionError> {
        let lr1a = LR1ABuilder::new(grammar).build();
        
        // let num_words  = word_count + 1; // +1 for eof
        let num_words  = grammar.max_word().map_or(0, |word| word + 1) + 1; // +1 for eof
        let num_vars   = grammar.var_count() - 1; // implicit start variable not needed in goto table
        let num_states = lr1a.states().len();
        
        let mut actions: Vec<Action>  = vec![Action::Invalid; num_words * num_states];
        let mut gotos: Vec<Option<usize>> = vec![None; num_vars * num_states];

        let reductions: Vec<_> = grammar.rules().enumerate().flat_map(|(i, rule)| {
            rule.alts().map(|alt| Reduction { var: i, count: alt.len() }).collect::<Vec<_>>()
        }).collect();

        for (i, state) in lr1a.states().iter().enumerate() {
            for item in &state.items {
                if !item.lr0_item.is_complete(grammar) {
                    let symbol = &item.lr0_item.symbol_at_dot(grammar).unwrap();
                    let index = match symbol {
                        Symbol::Terminal(a) => i * num_words + a + 1,
                        Symbol::Variable(_) => continue,
                    };
                    let action = actions.get_mut(index).unwrap();

                    // check for shift-reduce conflict
                    if let Action::Reduce(_) = action {
                        return Err(ConstructionError { 
                            state: i,
                            item: *item,
                            action1: *action,
                            action2: Action::Shift(state.next[symbol]),
                        });
                    } else {
                        *action = Action::Shift(state.next[symbol]);
                    }
                } else if reductions[item.lr0_item.alt].var < num_vars || item.lookahead.is_some() { // TODO: second check not necessary?
                    let index = i * num_words + item.lookahead.map_or(0, |a| a + 1);
                    let action = actions.get_mut(index).unwrap();

                    // check for any conflict
                    if let Action::Invalid = action {
                        *action = Action::Reduce(item.lr0_item.alt);
                    } else {
                        return Err(ConstructionError { 
                            state: i,
                            item: *item,
                            action1: *action,
                            action2: Action::Reduce(item.lr0_item.alt),
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

        Ok(Self {
            actions,
            gotos,
            reductions,
            word_count: num_words - 1, // subtract the eof symbol
            var_count: num_vars,
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