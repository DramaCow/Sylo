use super::{Command, LexAnalyzer};
use crate::re::{RegEx, DFA};

#[derive(Debug)]
pub struct LexDef {
    pub labels: Vec<String>,
    pub regexes: Vec<RegEx>,
    pub commands: Vec<Command>,
}

impl LexAnalyzer {
    #[must_use]
    pub fn compile(def: &LexDef) -> Self {
        let dfa = DFA::from(&def.regexes).minimize();
        
        let nrows = dfa.states().len() - 1; // excluding sink
        let mut table = vec![nrows; 256 * nrows];
        for (row, state) in dfa.states().iter().skip(1).enumerate() {
            for (&symbol, &dest) in &state.next {
                table[256 * row + symbol as usize] = dest - 1;
            }
        }
        
        let classes = dfa.states().iter().skip(1)
            .map(|state| state.class)
            .chain(vec![None]) // <-- sink states class
            .collect();
        
        LexAnalyzer {
            labels: def.labels.to_vec(),
            table,
            classes,
            commands: def.commands.to_vec(),
        }
    }
}