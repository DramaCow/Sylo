use std::iter::Copied;
use std::collections::hash_set;
use crate::langcore::cfg::{Grammar, Symbol};
use crate::langcore::lr::{LALR1A, LALR1ABuilder, LR0Item};
use super::{super::inner, NaiveLR1Table, Conflict, Action, ConstructionError, LR1TableConstructionStrategy};

pub struct LALR1;

impl LR1TableConstructionStrategy for LALR1 {
    fn construct<F: FnMut(Conflict) -> Result<Action, Conflict>>(grammar: &Grammar, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError> {
        let lalr1a = LALR1ABuilder::new(grammar).build();
        let builder = TableBuilder { grammar, lalr1a };
        inner::BuildLR1Table::build_lr1_table(&builder, grammar, conflict_resolution)
    }
}

// =================
// === INTERNALS ===
// =================

struct TableBuilder<'a> {
    grammar: &'a Grammar,
    lalr1a: LALR1A,
}

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