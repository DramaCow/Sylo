//! Parser generator internals.

pub mod re;
pub mod cfg;
// pub mod dfa;
pub mod lr1;
pub mod lex;
pub mod syn;

#[derive(Clone)]
pub enum Command {
    Skip,
    Emit,
}

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
    ParserDef,
    Parser,
    ParseError,
};