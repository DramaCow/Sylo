use crate::lang::re::RegEx;
use crate::lang::dfa::DFA;
use super::{LexAnalyzer, Command};

pub struct LexDef {
    pub regexes:  Vec<RegEx>,
    pub commands: Vec<Command>,
}

impl LexDef {
    #[must_use]
    pub fn compile(&self) -> LexAnalyzer {
        let dfa = DFA::from(&self.regexes).minimize();
        
        let nrows = dfa.states().len() - 1; // excluding sink
        let mut next = vec![nrows; 256 * nrows];
        for (row, state) in dfa.states().iter().skip(1).enumerate() {
            for (&symbol, &dest) in &state.next {
                next[256 * row + symbol as usize] = dest - 1;
            }
        }
        
        let classes = dfa.states().iter().skip(1)
            .map(|state| state.class)
            .chain(vec![None]) // <-- sink states class
            .collect();
        
        LexAnalyzer {
            next,
            classes,
            commands: self.commands.to_vec(),
        }
    }
}