//! Parser generator internals.

#[macro_use]
mod macros;

pub mod re;
pub mod cfg;
pub mod lr;

mod vocabulary;
pub use vocabulary::{
    Vocabulary,
};

mod lexer;
pub use lexer::{
    LexerDef,
    Lexer,
};

mod parser;
pub use parser::{
    Parser,
    ParserBuilder,
    Precedence,
    Associativity,
    // ParseError,
};

// =================
// === INTERNALS ===
// =================

#[cfg(test)]
mod tests;