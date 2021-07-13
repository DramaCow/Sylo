//! This is Sylo

#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
// #![warn(missing_docs)]

pub/*(crate)*/ mod utils;
pub mod lang;

pub mod lexer;
pub mod parser;
pub mod cst;

pub mod codegen;

#[macro_use]
mod macros;

// =================
// === INTERNALS ===
// =================

#[cfg(test)]
mod tests;