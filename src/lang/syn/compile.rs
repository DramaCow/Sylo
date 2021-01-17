use crate::lang::cfg::{Grammar, Symbol, lr1};
use super::{SynAnalyzer, Action, Reduction};

pub struct SynAnalyzerDef {
    pub grammar: Grammar,
    pub term_count: usize,
}

#[derive(Debug)]
pub enum CompileError {
    Conflict { state: usize, item: lr1::Item, },//action1: Action, action2: Action },
}

impl SynAnalyzerDef {
    /// # Errors
    pub fn compile(&self) -> Result<SynAnalyzer, CompileError> {
        let dfa = lr1::DFA::from(&self.grammar);
        
        let num_words  = self.term_count + 1; // +1 for eof
        let num_vars   = self.grammar.rule_count() - 1; // implicit start variable not needed in goto table
        let num_states = dfa.states.len();
        
        let mut actions: Vec<Action>      = vec![Action::Invalid; num_words * num_states];
        let mut gotos: Vec<Option<usize>> = vec![None; num_vars * num_states];

        for (i, state) in dfa.states.iter().enumerate() {
            for item in &state.items {
                if !self.grammar.is_complete(item) {
                    let symbol = &self.grammar.symbol_at_dot(item).unwrap();
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

        let reductions = self.grammar.rules().enumerate().flat_map(|(i, rule)| {
            rule.alts().map(|alt| Reduction{ var: i, count: alt.len() }).collect::<Vec<_>>()
        }).collect();

        Ok(SynAnalyzer {
            actions,
            gotos,
            reductions,
            term_count: self.term_count,
            var_count: num_vars,
        })
    }
}