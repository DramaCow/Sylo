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
    LR1TableConstructionStrategy,
    //
    with_conflict_resolution,
    construct,
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

#[cfg(test)]
mod tests;