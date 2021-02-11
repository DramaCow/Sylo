#![allow(non_snake_case)]

pub use self::compile::{SynDef, CompileError};
pub use self::parse::{Instruction, Parse, ParseError};

#[derive(Debug)]
pub struct SynAnalyzer {
    actions:    Vec<Action>,        /// lookup what action to perform given state and word
    gotos:      Vec<Option<usize>>, /// lookup what state should be transitioned to after reduction
    reductions: Vec<Reduction>,     // alt --> rule and number of symbols
    term_count: usize,
    var_count:  usize,
}

impl SynAnalyzer {
    #[must_use]
    pub fn parse<T: IntoIterator<Item=usize>>(&self, words: T) -> Parse<T::IntoIter> {
        Parse::new(&self, words.into_iter())
    }
}

// =================
// === INTERNALS ===
// =================

mod compile;
mod parse;

#[derive(Debug, Clone, Copy)]
struct Reduction {
    var: usize,
    count: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Action {
    Invalid,
    Accept,
    Shift(usize),
    Reduce(usize),
}

impl SynAnalyzer {
    fn action(&self, state: usize, word: Option<usize>) -> Action {
        word.map_or_else(
            || self.actions[state * (self.term_count + 1)],
            |a| self.actions[state * (self.term_count + 1) + a + 1]
        )
    }

    fn goto(&self, state: usize, var: usize) -> Option<usize> {
        self.gotos[state * self.var_count + var]
    }
}

#[cfg(test)]
mod tests;