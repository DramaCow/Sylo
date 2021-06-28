use super::LRkItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LR0Item {
    pub production: usize, // index of production
    pub pos: usize,        // index position of dot in production RHS
}

impl LRkItem for LR0Item {
    fn production(&self) -> usize {
        self.production
    }

    fn pos(&self) -> usize {
        self.pos
    }
}

impl LR0Item {
    #[must_use]
    pub fn new(production: usize, pos: usize) -> Self {
        Self { production, pos }
    }
}