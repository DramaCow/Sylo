use std::collections::{BTreeSet, HashMap};

use super::{Grammar, Symbol};

/// LR(1) item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Item {
    pub rule: usize,              // index of rule
    pub alt:  usize,              // index of alt
    pub pos:  usize,              // index of dot in alt
    // If all of alt is present in context (LHS), and the suceeding
    // symbol is successor, then lhs can be reduced to rule (RHS).
    pub successor: Option<usize>, // class of successor terminal
}

pub struct DFA {
    pub states: Vec<State>,
}

pub struct State {
    pub items: BTreeSet<Item>,
    pub next: HashMap<Symbol, usize>,
}

impl From<&Grammar> for DFA {
    fn from(grammar: &Grammar) -> Self {
        DFABuilder::new(grammar).build()
    }
}

// =================
// === INTERNALS ===
// =================

impl Grammar {
    pub(crate) fn is_complete(&self, item: &Item) -> bool {
        item.pos >= self.alt(item.alt).len()
    }

    pub(crate) fn symbol_at_dot(&self, item: &Item) -> Option<Symbol> {
        let alt = &self.alt(item.alt);
        if item.pos < alt.len() {
            Some(alt[item.pos])
        } else {
            None
        }
    }

    pub(crate) fn symbol_after_dot(&self, item: &Item) -> Option<Symbol> {
        let alt = &self.alt(item.alt);
        if item.pos + 1 < alt.len() {
            Some(alt[item.pos+1])
        } else {
            None
        }
    }
}

mod builder;
use builder::DFABuilder;