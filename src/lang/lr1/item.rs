use std::fmt;
use crate::lang::cfg::{Grammar, Symbol};

/// LR(1) item.
// If all of alt is present in context (LHS), and the suceeding
// symbol is successor, then lhs can be reduced to rule (RHS).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Item {
    pub rule: usize,              // index of rule
    pub alt: usize,               // index of alt
    pub pos: usize,               // (index) position of dot in alt
    pub successor: Option<usize>, // class of successor terminal
}

impl Item {
    #[must_use]
    pub fn is_complete(&self, grammar: &Grammar) -> bool {
        self.pos >= grammar.alt(self.alt).len()
    }

    #[must_use]
    pub fn symbol_at_dot(&self, grammar: &Grammar) -> Option<Symbol> {
        grammar.alt(self.alt).get(self.pos).cloned()
    }

    #[must_use]
    pub fn symbol_after_dot(&self, grammar: &Grammar) -> Option<Symbol> {
        grammar.alt(self.alt).get(self.pos + 1).cloned()
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