//! Sylo is currently in early development and is highly unstable.
//! Use is not yet recommended.

#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
// #![warn(missing_docs)]

pub mod utils;

pub mod langcore;

#[macro_use]
mod parsing;
pub use parsing::{lexer, parser, cst, codegen, re};

// mod bindings {
//     #![allow(non_upper_case_globals)]
//     #![allow(non_camel_case_types)]
//     #![allow(non_snake_case)]
//     #![allow(deref_nullptr)]
//     #![allow(dead_code)]

//     include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
// }