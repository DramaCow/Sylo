use crate::lang::cfg::{Grammar, Symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LR0Item {
    pub alt: usize, // index of alt
    pub pos: usize, // (index) position of dot in alt
}

impl LR0Item {
    #[must_use]
    pub fn new(alt: usize, pos: usize) -> Self {
        Self { alt, pos }
    }

    #[must_use]
    pub fn is_kernel_item(&self, grammar: &Grammar) -> bool {
        // is the start rule or dot not at start
        self.alt == grammar.alt_count() - 1 || self.pos > 0
    }

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