mod table;
pub use self::table::{
    Action,
    Reduction,
    LR1Table,
    NaiveLR1Table,
};

mod construct;
pub use self::construct::{
    ConstructionError,
    Conflict,
    LR1TableConstruction,
    LR1TableBuilderStrategy,
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