//! Parser generator internals.

pub mod re;
pub mod cfg;
pub mod lex;
pub mod syn;

#[derive(Clone)]
pub enum Command {
    Skip,
    Emit,
}

mod lexer;

mod parser;
pub use parser::{
    ParserDef,
    Parser,
    ParseError,
};