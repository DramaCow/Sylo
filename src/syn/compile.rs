use super::{Action, Reduction, SynAnalyzer, SynDef};
use crate::cfg::{Symbol, lr1};

#[derive(Debug)]
pub enum ConstructionError {
    Conflict { state: usize, item: lr1::Item, action1: Action, action2: Action },
    // ShiftReduceConflict,
    // ReduceReduceConflict,
}

impl SynAnalyzer {
    /// # Errors
    pub fn try_compile(def: &SynDef) -> Result<Self, ConstructionError> {
        let dfa = lr1::DFA::from(&def.grammar);
        
        let num_vars = def.grammar.rule_count() - 1; // implicit start variable not needed in goto table
        let num_words = def.grammar.termcount + 1; // +1 for eof
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
                        return Err(ConstructionError::Conflict { 
                            state: i,
                            item: *item,
                            action1: *action,
                            action2: Action::Shift(state.next[symbol]),
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
                        return Err(ConstructionError::Conflict { 
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

        let reductions = def.grammar.rules().enumerate().flat_map(|(i, rule)| {
            rule.alts().map(|alt| Reduction{ var: i, count: alt.len() }).collect::<Vec<_>>()
        }).collect();

        Ok(Self {
            labels: def.labels.to_vec(),
            actions,
            gotos,
            reductions,
            termcount: def.grammar.termcount,
            commands: def.commands.to_vec(),
        })
    }
}