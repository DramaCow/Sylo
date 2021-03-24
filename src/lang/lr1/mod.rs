//! Canonical LR(1) analysis tools.

mod item;
pub use self::item::Item;

mod lr1a;
pub use self::lr1a::LR1A;

mod array_parsing_table;
pub use self::array_parsing_table::{
    ArrayParsingTable,
    ConstructionError,
};

#[cfg(test)]
mod tests;