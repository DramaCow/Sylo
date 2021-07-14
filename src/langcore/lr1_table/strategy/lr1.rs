use std::iter::{Once, once};
use crate::langcore::cfg::{Grammar, Symbol};
use crate::langcore::lr::{LR1A, LR1ABuilder, LR1Item};
use super::{inner, NaiveLR1Table, Conflict, Action, ConstructionError, LR1TableConstructionStrategy};

pub struct LR1;

impl LR1TableConstructionStrategy for LR1 {
    fn construct<F: FnMut(Conflict) -> Result<Action, Conflict>>(self, grammar: &Grammar, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError> {
        let lr1a = LR1ABuilder::new(grammar).build();
        let builder = TableBuilder { grammar, lr1a };
        inner::BuildLR1Table::build(&builder, grammar, conflict_resolution)
    }
}

// =================
// === INTERNALS ===
// =================

struct TableBuilder<'a> {
    grammar: &'a Grammar,
    lr1a: LR1A,
}

impl inner::BuildLR1Table<'_, LR1Item, Once<Option<usize>>> for TableBuilder<'_> {
    fn state_count(&self) -> usize {
        self.lr1a.states().len()
    }

    fn items(&self, state: usize) -> &[LR1Item] {
        &self.lr1a.states()[state].items
    }

    fn transition(&self, state: usize, symbol: Symbol) -> Option<usize> {
        self.lr1a.states()[state].next.get(&symbol).copied()
    }

    fn lookaheads(&self, _: usize, item: &LR1Item) -> Once<Option<usize>> {
        once(item.lookahead)
    }

    fn production(&self, item: &LR1Item) -> usize {
        item.lr0_item.production
    }

    fn is_complete(&self, item: &LR1Item) -> bool {
        item.lr0_item.is_complete(self.grammar)
    }

    fn symbol_at_dot(&self, item: &LR1Item) -> Option<Symbol> {
        item.lr0_item.symbol_at_dot(self.grammar)
    }
}
