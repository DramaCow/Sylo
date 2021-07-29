use std::iter::Copied;
use std::collections::hash_set;
use crate::langcore::cfg::{Grammar, Symbol};
use crate::langcore::lr::{LALR1A, LALR1ABuilder, LR0Item};
use super::{super::inner};

pub struct LALR1;

impl<'a> inner::InnerStrategy<'a> for LALR1 {
    type Builder = TableBuilder<'a>;
    
    fn builder(&self, grammar: &'a Grammar) -> Self::Builder {
        let lalr1a = LALR1ABuilder::new(grammar).build();
        TableBuilder { grammar, lalr1a }
    }
}

pub struct TableBuilder<'a> {
    grammar: &'a Grammar,
    lalr1a: LALR1A,
}

// =================
// === INTERNALS ===
// =================

impl inner::ItemSets for TableBuilder<'_> {
    type Item = LR0Item;

    fn state_count(&self) -> usize {
        self.lalr1a.states().len()
    }

    fn items(&self, state: usize) -> &[Self::Item] {
        &self.lalr1a.states()[state].items
    }

    fn transition(&self, state: usize, symbol: Symbol) -> Option<usize> {
        self.lalr1a.states()[state].next.get(&symbol).copied()
    }

    fn production(&self, item: &Self::Item) -> usize {
        item.production
    }

    fn is_complete(&self, item: &Self::Item) -> bool {
        item.is_complete(self.grammar)
    }

    fn symbol_at_dot(&self, item: &Self::Item) -> Option<Symbol> {
        item.symbol_at_dot(self.grammar)
    }
}

impl<'a> inner::Lookaheads<'a> for TableBuilder<'_> {
    type Output = Copied<hash_set::Iter<'a, Option<usize>>>;

    fn lookaheads(&'a self, state: usize, item: &Self::Item) -> Self::Output {
        self.lalr1a.lookaheads(state, item.production).iter().copied()
    }
}