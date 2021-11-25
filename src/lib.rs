//! Sylo is currently in early development and is highly unstable.
//! Use is not yet recommended.

#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
// #![warn(missing_docs)]

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod re_parser;

#[macro_use]
mod macros;

// =================
// === INTERNALS ===
// =================

pub(crate) use regex_deriv as re;
pub(crate) use lr_parsing_tools as lr;

#[cfg(test)]
mod tests;