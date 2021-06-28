use super::{LRkItem, LR0Item};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LR1Item {
    pub lr0_item: LR0Item,
    pub lookahead: Option<usize>, // class of lookahead terminal
}

impl LRkItem for LR1Item {
    fn production(&self) -> usize {
        self.lr0_item.production
    }

    fn pos(&self) -> usize {
        self.lr0_item.pos
    }
}

impl LR1Item {
    #[must_use]
    pub fn new(alt: usize, pos: usize, lookahead: Option<usize>) -> Self {
        Self {
            lr0_item: LR0Item::new(alt, pos),
            lookahead,
        }
    }
}