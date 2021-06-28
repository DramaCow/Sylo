use crate::lang::cfg::{Grammar, Symbol};

pub trait LRkItem {
    fn production(&self) -> usize;
    fn pos(&self) -> usize;

    /// I.e. is the start rule or dot *not* at the start.
    #[must_use]
    fn is_kernel_item(&self, grammar: &Grammar) -> bool {
        self.production() == grammar.production_count() - 1 || self.pos() > 0
    }

    /// I.e. dot is past the end. 
    #[must_use]
    fn is_complete(&self, grammar: &Grammar) -> bool {
        self.pos() >= grammar.alt(self.production()).len()
    }

    #[must_use]
    fn symbol_at_dot(&self, grammar: &Grammar) -> Option<Symbol> {
        grammar.alt(self.production()).get(self.pos()).copied()
    }

    #[must_use]
    fn symbol_after_dot(&self, grammar: &Grammar) -> Option<Symbol> {
        grammar.alt(self.production()).get(self.pos() + 1).copied()
    }
}