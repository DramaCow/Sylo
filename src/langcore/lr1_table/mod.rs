mod table;
use self::table::Reduction;
pub use self::table::{
    Action,
    LR1Table,
    NaiveLR1Table,
};

mod construct;
pub use self::construct::{
    ConstructionError,
    Conflict,
    LR1TableConstruction,
    LR1TableBuilderStrategy,
    LongestCommonPrecedingSubpath,
};

mod parse;
pub use self::parse::{
    Event,
    Parse,
    ParseError,
};

pub mod strategy;

// =================
// === INTERNALS ===
// =================

mod inner;

#[cfg(test)]
mod tests;