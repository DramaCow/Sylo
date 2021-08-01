#![allow(clippy::items_after_statements)]

pub mod lexer;
pub mod parser;
pub mod cst;
pub mod codegen;

pub mod re;

#[macro_use]
mod macros;

// =================
// === INTERNALS ===
// =================

#[cfg(test)]
mod tests;