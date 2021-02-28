use crate::lang::cfg::{Grammar, Symbol};
use crate::lang::lr1::{self, LR1A};
use super::{SynAnalyzer, Action, Reduction};

pub struct SynDef {
    pub grammar: Grammar,
    pub word_count: usize,
}

#[derive(Debug)]
pub struct CompileError {
    state: usize,
    item: lr1::Item,
    action1: Action,
    action2: Action,
}

impl SynDef {
    /// # Errors
    pub fn compile(&self) -> Result<SynAnalyzer, CompileError> {
        let lr1a = LR1A::from(&self.grammar);
        
        let num_words  = self.word_count + 1; // +1 for eof
        let num_vars   = self.grammar.rule_count() - 1; // implicit start variable not needed in goto table
        let num_states = lr1a.states().len();
        
        let mut actions: Vec<Action>      = vec![Action::Invalid; num_words * num_states];
        let mut gotos: Vec<Option<usize>> = vec![None; num_vars * num_states];

        for (i, state) in lr1a.states().iter().enumerate() {
            for item in &state.items {
                if !item.is_complete(&self.grammar) {
                    let symbol = &item.symbol_at_dot(&self.grammar).unwrap();
                    let index = match symbol {
                        Symbol::Terminal(a) => i * num_words + a + 1,
                        Symbol::Variable(_) => continue,
                    };
                    let action = actions.get_mut(index).unwrap();

                    // check for shift-reduce conflict
                    if let Action::Reduce(_) = action {
                        return Err(CompileError { 
                            state: i,
                            item: *item,
                            action1: *action,
                            action2: Action::Shift(state.next[symbol]),
                        });
                    } else {
                        *action = Action::Shift(state.next[symbol]);
                    }
                } else if item.rule < num_vars || item.successor.is_some() { // TODO: second check not necessary?
                    let index = i * num_words + item.successor.map_or(0, |a| a + 1);
                    let action = actions.get_mut(index).unwrap();

                    // check for any conflict
                    if let Action::Invalid = action {
                        *action = Action::Reduce(item.alt);
                    } else {
                        return Err(CompileError { 
                            state: i,
                            item: *item,
                            action1: *action,
                            action2: Action::Reduce(item.alt),
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

        let reductions = self.grammar.rules().enumerate().flat_map(|(i, rule)| {
            rule.alts().map(|alt| Reduction{ var: i, count: alt.len() }).collect::<Vec<_>>()
        }).collect();

        Ok(SynAnalyzer {
            actions,
            gotos,
            reductions,
            word_count: self.word_count,
            var_count: num_vars,
        })
    }
}