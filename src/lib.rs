//! Sylo is currently in early development and is highly unstable.
//! Use is not yet recommended.

#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
// #![warn(missing_docs)]

#[macro_use] extern crate lalrpop_util;

lalrpop_mod!(pub meta);

pub mod tok;
pub mod ast;
pub mod repr;
pub mod lexer;
pub mod parser;

#[macro_use]
mod macros;

// =================
// === INTERNALS ===
// =================

#[cfg(test)]
mod tests;