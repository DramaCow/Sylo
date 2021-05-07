pub mod compile;

mod lexer;
pub use lexer::{
    Lexer,
    LexerBuilder,
};

mod parser;
pub use parser::{
    Parser,
    ParserBuilder,
    Precedence,
    Associativity,
    // ParseError,
};

mod cst;
pub use self::cst::{
    CST,
    CSTBuilder,
};

#[macro_use]
mod macros;

// =================
// === INTERNALS ===
// =================

#[cfg(test)]
mod tests;