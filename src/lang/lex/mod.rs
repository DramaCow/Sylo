use crate::lang::re::{RegEx, DFA};

pub use self::parse::{Token, Parse, ParseError};

#[derive(Debug, Clone)]
pub enum Command {
    Emit,
    Skip,
}

pub struct LexAnalyzerDef {
    pub labels:   Vec<String>,
    pub regexes:  Vec<RegEx>,
    pub commands: Vec<Command>,
}

pub struct LexAnalyzer {
    labels:   Vec<String>,
    table:    Vec<usize>,
    classes:  Vec<Option<usize>>,
    commands: Vec<Command>,
}

impl LexAnalyzer {
    #[must_use]
    pub fn compile(def: &LexAnalyzerDef) -> Self {
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

    #[must_use]
    pub fn parse<'a>(&'a self, text: &'a str) -> Parse<'a> {
        Parse::new(&self, text)
    }
}

// =================
// === INTERNALS ===
// =================

mod parse;

#[cfg(test)]
mod tests;