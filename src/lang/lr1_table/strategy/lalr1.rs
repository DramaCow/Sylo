use std::iter::Copied;
use std::collections::hash_set;
use crate::lang::cfg::{Grammar, Symbol};
use crate::lang::lr::{LALR1A, LALR1ABuilder, LR0Item};
use super::{inner, NaiveLR1Table, Conflict, Action, ConstructionError, LR1TableConstructionStrategy};

pub struct LALR1;

impl LR1TableConstructionStrategy for LALR1 {
    fn construct<F: FnMut(Conflict) -> Result<Action, Conflict>>(self, grammar: &Grammar, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError> {
        let lalr1a = LALR1ABuilder::new(grammar).build();
        let builder = TableBuilder { grammar, lalr1a };
        inner::BuildLR1Table::build(&builder, grammar, conflict_resolution)
    }
}

// =================
// === INTERNALS ===
// =================

struct TableBuilder<'a> {
    grammar: &'a Grammar,
    lalr1a: LALR1A,
}

type Iter<'a> = Copied<hash_set::Iter<'a, Option<usize>>>;

impl<'a> inner::BuildLR1Table<'a, LR0Item, Iter<'a>> for TableBuilder<'a> {
    fn state_count(&self) -> usize {
        self.lalr1a.states().len()
    }

    fn items(&self, state: usize) -> &[LR0Item] {
        &self.lalr1a.states()[state].items
    }

    fn transition(&self, state: usize, symbol: Symbol) -> Option<usize> {
        self.lalr1a.states()[state].next.get(&symbol).copied()
    }

    fn production(&self, item: &LR0Item) -> usize {
        item.production
    }

    fn is_complete(&self, item: &LR0Item) -> bool {
        item.is_complete(self.grammar)
    }

    fn symbol_at_dot(&self, item: &LR0Item) -> Option<Symbol> {
        item.symbol_at_dot(self.grammar)
    }

    fn lookaheads(&'a self, state: usize, item: &LR0Item) -> Iter<'a> {
        self.lalr1a.lookaheads(state, item.production).iter().copied()
    }
}

pub trait MyTrait<I: IntoIterator<Item = usize>> {
    fn iter(&self) -> I;
}