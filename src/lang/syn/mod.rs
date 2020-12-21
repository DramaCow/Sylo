use crate::lang::cfg::Grammar;

pub use self::compile::ConstructionError;
pub use self::parse::{Action, SynParse, SynParseError};

#[derive(Debug, Clone)]
pub enum SynCommand {
    Emit,
    Skip,
}

pub struct SynAnalyzerDef {
    pub labels: Vec<String>,
    pub grammar: Grammar,
    pub commands: Vec<SynCommand>,
}

#[derive(Debug)]
pub struct SynAnalyzer {
    labels:       Vec<String>,
    actions:      Vec<SynAction>,
    gotos:        Vec<Option<usize>>,
    reductions:   Vec<Reduction>,     // alt --> rule + number of symbols
    term_count:   usize,
    pub commands: Vec<SynCommand>,
}

impl SynAnalyzer {
    #[must_use]
    pub fn parse<T: IntoIterator<Item=usize>>(&self, words: T) -> SynParse<T::IntoIter> {
        SynParse::new(&self, words.into_iter())
    }
}

// =================
// === INTERNALS ===
// =================

mod parse;
mod compile;

impl SynAnalyzer {  
    fn var_count(&self) -> usize {
        self.commands.len()
    }
}

#[derive(Debug, Clone, Copy)]
struct Reduction {
    var: usize,
    count: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SynAction {
    Invalid,
    Accept,
    Shift(usize),
    Reduce(usize),
}

impl SynAnalyzer {
    fn action(&self, state: usize, word: Option<usize>) -> SynAction {
        word.map_or(
            self.actions[state * (self.term_count + 1)],
            |a| self.actions[state * (self.term_count + 1) + a + 1]
        )
    }

    fn goto(&self, state: usize, var: usize) -> Option<usize> {
        self.gotos[state * self.var_count() + var]
    }
}

#[cfg(test)]
mod tests;