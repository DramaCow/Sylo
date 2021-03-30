use crate::lang::cfg::{Grammar, Symbol};

/// LR(1) item.
// If all of alt is present in context (LHS), and the suceeding
// symbol is lookahead, then lhs can be reduced to rule (RHS).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Item {
    pub alt: usize,               // index of alt
    pub pos: usize,               // (index) position of dot in alt
    pub lookahead: Option<usize>, // class of lookahead terminal
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