#![allow(clippy::items_after_statements)]

pub mod lexer;
pub mod parser;
pub mod cst;
pub mod codegen;

#[allow(non_camel_case_types)]
#[allow(unused_comparisons)]
#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(unused_mut)]
#[allow(clippy::match_same_arms)]
#[allow(clippy::needless_return)]
#[allow(clippy::unnecessary_wraps)]
pub mod re;

#[macro_use]
mod macros;

// =================
// === INTERNALS ===
// =================

#[cfg(test)]
mod tests;