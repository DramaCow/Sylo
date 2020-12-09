#![allow(non_snake_case)]

pub mod parse;
pub mod parse_tree;

mod compile;

use crate::cfg::Grammar;
use self::parse::{Parse, ParseStep, ParseError};
use self::parse_tree::{ParseTree, ParseTreeBuilder};

pub use self::compile::ConstructionError;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Invalid,
    Accept,
    Shift(usize),
    Reduce(usize),
}

#[derive(Debug)]
pub struct Reduction {
    var: usize,
    count: usize,
}

#[derive(Debug, Clone)]
pub enum Command {
    Emit,
    Skip, // propagate
}

pub struct SynDef {
    pub labels: Vec<String>,
    pub grammar: Grammar,
    pub commands: Vec<Command>,
}

#[derive(Debug)]
pub struct SynAnalyzer {
    labels:     Vec<String>,
    actions:    Vec<Action>,
    gotos:      Vec<Option<usize>>,
    reductions: Vec<Reduction>,     // alt --> rule + number of symbols
    termcount:  usize,
    commands:   Vec<Command>,
}

impl SynAnalyzer {
    #[must_use]
    pub fn parse<T: IntoIterator<Item=usize>>(&self, words: T) -> Parse<T::IntoIter> {
        Parse::new(&self, words.into_iter())
    }

    /// # Errors
    pub fn parse_tree<T: IntoIterator<Item=usize>>(&self, words: T) -> Result<ParseTree, ParseError> {
        let mut builder = ParseTreeBuilder::new();

        for step in self.parse(words) {
            match step? {
                ParseStep::Leaf { word, index } => builder.leaf(word, index),
                ParseStep::Branch { var, num_children } => builder.branch(var, num_children),
                ParseStep::List { num_children } => builder.list(num_children),
            }
        }

        Ok(builder.build())
    }

    #[must_use]
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    #[must_use]
    pub fn gotos(&self) -> &[Option<usize>] {
        &self.gotos
    }

    #[must_use]
    pub fn var_count(&self) -> usize {
        self.commands.len()
    }

    #[must_use]
    pub fn term_count(&self) -> usize {
        self.termcount
    }
}

// =================
// === INTERNALS ===
// =================

impl SynAnalyzer {
    fn action(&self, state: usize, word: Option<usize>) -> &Action {
        word.map_or(
            &self.actions[state * (self.termcount + 1)],
            |a| &self.actions[state * (self.termcount + 1) + a + 1]
        )
    }

    fn goto(&self, state: usize, var: usize) -> &Option<usize> {
        &self.gotos[state * self.var_count() + var]
    }
}

#[cfg(test)]
mod tests;