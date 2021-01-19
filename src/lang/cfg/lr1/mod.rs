use std::collections::{BTreeSet, HashMap};
use std::fmt;

use super::{Grammar, Symbol};

/// LR(1) item.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if let Some(delta) = self.successor {
            f.write_str(&format!("(alt: {}, pos: {}) --> {}", self.alt, self.pos, delta))
        } else {
            f.write_str(&format!("(alt: {}, pos: {}) --> \u{03b5}", self.alt, self.pos))
        }
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