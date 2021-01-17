pub use self::compile::LexAnalyzerDef;
pub use self::parse::{Token, Parse, ParseError};

#[derive(Debug, Clone)]
pub enum Command {
    Emit,
    Skip,
}

pub struct LexAnalyzer {
    table:    Vec<usize>,
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