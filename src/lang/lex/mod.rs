pub use self::compile::LexDef;
pub use self::parse::{Token, Parse, ParseError};
use super::Command;

pub struct LexAnalyzer {
    next:     Vec<usize>,
    classes:  Vec<Option<usize>>,
    commands: Vec<Command>,
}

impl LexAnalyzer {
    #[must_use]
    pub fn parse<'a>(&'a self, text: &'a str) -> Parse<'a> {
        Parse::new(&self, text)
    }
}

// =================
// === INTERNALS ===
// =================

mod compile;
mod parse;

#[cfg(test)]
mod tests;