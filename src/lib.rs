#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]

#[macro_use]
mod macros;

pub mod collections;
pub mod iter;

pub mod lang;
pub mod cst;

pub mod debug;