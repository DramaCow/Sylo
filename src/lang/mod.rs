//! Parser generator internals.

#[macro_use]
mod macros;

pub mod re;
pub mod cfg;

pub mod dfa;
// pub mod lr0;
pub mod lr1;

pub mod lr;
pub mod lex;

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