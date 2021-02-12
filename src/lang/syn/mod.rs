#![allow(non_snake_case)]

pub use self::compile::{SynDef, CompileError};
pub use self::parse::{Instruction, Parse, ParseError};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Invalid,
    Accept,
    Shift(usize),
    Reduce(usize),
}

#[derive(Debug)]
pub struct SynAnalyzer {
    actions:    Vec<Action>,        /// lookup what action to perform given state and word
    gotos:      Vec<Option<usize>>, /// lookup what state should be transitioned to after reduction
    reductions: Vec<Reduction>,     // alt --> rule and number of symbols
    word_count: usize,
    var_count:  usize,
}

impl SynAnalyzer {
    #[must_use]
    pub fn parse<T: IntoIterator<Item=usize>>(&self, words: T) -> Parse<T::IntoIter> {
        Parse::new(&self, words.into_iter())
    }

    #[must_use]
    pub fn action(&self, state: usize, word: Option<usize>) -> Action {
        word.map_or_else(
            || self.actions[state * (self.word_count + 1)],
            |a| self.actions[state * (self.word_count + 1) + a + 1]
        )
    }

    #[must_use]
    pub fn goto(&self, state: usize, var: usize) -> Option<usize> {
        self.gotos[state * self.var_count + var]
    }

    // #[must_use]
    // pub fn actions_row(&self, state: usize) -> &[Action] {
    //     let low = state * (self.word_count + 1);
    //     let high = (state + 1) * (self.word_count + 1);
    //     &self.actions[low..high]
    // }

    // #[must_use]
    // pub fn gotos_row(&self, state: usize) -> &[Option<usize>] {
    //     let low = state * self.var_count;
    //     let high = (state + 1) * self.var_count;
    //     &self.gotos[low..high]
    // }
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

#[cfg(test)]
mod tests;