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

mod parse;
pub use self::parse::{
    Action,
    Reduction,
    LRTable,
    Event,
    Parse,
    ParseError,
};

mod naive;
pub use self::naive::{
    ConstructionError,
    Conflict,
    NaiveLRTable,
};

// =================
// === INTERNALS ===
// =================

mod automaton;
use self::automaton::BuildItemSets;

#[cfg(test)]
mod tests;