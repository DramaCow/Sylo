use std::iter::{Once, once};
use crate::langcore::cfg::{Grammar, Symbol};
use crate::langcore::lr::{LR1A, LR1Item};
use super::{super::inner, NaiveLR1Table, Conflict, Action, ConstructionError, LR1TableConstructionStrategy};

pub struct LR1;

impl LR1TableConstructionStrategy for LR1 {
    fn construct<F: FnMut(Conflict) -> Result<Action, Conflict>>(grammar: &Grammar, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError> {
        let lr1a = LR1A::new(grammar);
        let builder = TableBuilder { grammar, lr1a };
        inner::BuildLR1Table::build_lr1_table(&builder, grammar, conflict_resolution)
    }
}

// =================
// === INTERNALS ===
// =================

struct TableBuilder<'a> {
    grammar: &'a Grammar,
    lr1a: LR1A,
}

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

    fn is_complete(&self, item: &Self::Item) -> bool {
        item.lr0_item.is_complete(self.grammar)
    }

    fn symbol_at_dot(&self, item: &Self::Item) -> Option<Symbol> {
        item.lr0_item.symbol_at_dot(self.grammar)
    }    
}

impl<'a> inner::Lookaheads<'a> for TableBuilder<'_> {
    type Output = Once<Option<usize>>;

    fn lookaheads(&self, _: usize, item: &Self::Item) -> Self::Output {
        once(item.lookahead)
    }
}
