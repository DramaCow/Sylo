use crate::lang::cfg::{Grammar, Symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LR0Item {
    pub production: usize, // index of production
    pub pos: usize,        // index position of dot in production RHS
}

impl LR0Item {
    #[must_use]
    pub fn new(production: usize, pos: usize) -> Self {
        Self { production, pos }
    }

    /// I.e. is the start rule or dot *not* at the start.
    #[must_use]
    pub fn is_kernel_item(&self, grammar: &Grammar) -> bool {
        self.production == grammar.production_count() - 1 || self.pos > 0
    }

    /// I.e. dot is past the end. 
    #[must_use]
    pub fn is_complete(&self, grammar: &Grammar) -> bool {
        self.pos >= grammar.alt(self.production).len()
    }

    #[must_use]
    pub fn symbol_at_dot(&self, grammar: &Grammar) -> Option<Symbol> {
        grammar.alt(self.production).get(self.pos).copied()
    }

    #[must_use]
    pub fn symbol_after_dot(&self, grammar: &Grammar) -> Option<Symbol> {
        grammar.alt(self.production).get(self.pos + 1).copied()
    }
}