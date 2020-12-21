use crate::lang::re::RegEx;

pub use self::parse::{Token, LexParse, LexParseError};

#[derive(Debug, Clone)]
pub enum LexCommand {
    Emit,
    Skip,
}

#[derive(Debug)]
pub struct LexAnalyzerDef {
    pub labels: Vec<String>,
    pub regexes: Vec<RegEx>,
    pub commands: Vec<LexCommand>,
}

pub struct LexAnalyzer {
    labels:   Vec<String>,
    table:    Vec<usize>,
    classes:  Vec<Option<usize>>,
    commands: Vec<LexCommand>,
}

impl LexAnalyzer {
    #[must_use]
    pub fn parse<'a>(&'a self, text: &'a str) -> LexParse<'a> {
        LexParse::new(&self, text)
    }
}

// =================
// === INTERNALS ===
// =================

mod compile;
mod parse;

#[cfg(test)]
mod tests;