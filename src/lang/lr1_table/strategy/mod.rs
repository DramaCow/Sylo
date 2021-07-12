mod lr1;
pub use self::lr1::LR1;

mod lalr1;
pub use self::lalr1::LALR1;

// =================
// === INTERNALS ===
// =================

use super::{
    Action,
    Reduction,
    Conflict,
    ConstructionError,
    NaiveLR1Table,
    LR1TableConstructionStrategy
};

mod inner;