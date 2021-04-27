//! This is Sylo

#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
// #![warn(missing_docs)]

pub(crate) mod utils;

#[macro_use]
pub mod lang;
pub mod syntax;