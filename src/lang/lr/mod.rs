mod lr0_item;
mod lr1_item;
pub use self::{
    lr0_item::LR0Item,
    lr1_item::LR1Item,
};

mod lr0a;
mod lalr1a;
mod lr1a;
pub use self::{
    lr0a::{LR0A, LR0ABuilder},
    lalr1a::{LALR1ABuilder, NonterminalTransition, StateReductionPair},
    lr1a::{LR1A, LR1ABuilder},
};

mod lr1_table;
pub use self::lr1_table::{
    Action,
    Reduction,
    LR1Table,
};

mod parse;
pub use self::parse::{
    Event,
    Parse,
    ParseError,
};

mod construction;
pub use self::construction::{
    ConstructionError,
    Conflict,
    NaiveLR1Table,
};

// =================
// === INTERNALS ===
// =================

mod inner;
use self::inner::{
    BuildItemSets,
    BuildLR1Table,
};

#[cfg(test)]
mod tests;