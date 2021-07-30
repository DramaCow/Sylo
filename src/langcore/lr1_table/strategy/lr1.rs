use std::iter::{Once, once};
use crate::langcore::cfg::{Grammar, Symbol};
use crate::langcore::lr::{LR1A, LR1Item};
use super::{super::inner};

pub struct LR1;

// impl<'a> Strategy<TableBuilder<'a>> for LR1 {}
impl<'a> inner::InnerStrategy<'a> for LR1 {
    type Builder = TableBuilder<'a>;

    fn builder(&self, grammar: &'a Grammar) -> Self::Builder {
        let lr1a = LR1A::new(grammar);
        TableBuilder { grammar, lr1a }
    }
}

pub struct TableBuilder<'a> {
    grammar: &'a Grammar,
    lr1a: LR1A,
}

// =================
// === INTERNALS ===
// =================

impl inner::ItemSets for TableBuilder<'_> {
    type Item = LR1Item;

    fn state_count(&self) -> usize {
        self.lr1a.states().len()
    }

    fn items(&self, state: usize) -> &[Self::Item] {
        &self.lr1a.states()[state].items
    }

    fn transition(&self, state: usize, symbol: Symbol) -> Option<usize> {
        self.lr1a.states()[state].next.get(&symbol).copied()
    }

    fn production(&self, item: &Self::Item) -> usize {
        item.lr0_item.production
    }

    fn pos(&self, item: &Self::Item) -> usize {
        item.lr0_item.pos
    }

    fn is_complete(&self, item: &Self::Item) -> bool {
        item.lr0_item.is_complete(self.grammar)
    }

    fn symbol_at_dot(&self, item: &Self::Item) -> Option<Symbol> {
        item.lr0_item.symbol_at_dot(self.grammar)
    }    
}

impl inner::Lookaheads<'_> for TableBuilder<'_> {
    type Output = Once<Option<usize>>;

    fn lookaheads(&self, _: usize, item: &Self::Item) -> Self::Output {
        once(item.lookahead)
    }
}
