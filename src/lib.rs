#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod macros;

pub mod re;
pub mod cfg;
pub mod lex;
pub mod syn;
pub mod iter;

pub mod parser;

pub mod debug;