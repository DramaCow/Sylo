#![allow(non_snake_case)]

use std::collections::BTreeSet;
use crate::lang::cfg::{Grammar, Symbol};
use super::{LR0Item, LR0A, State};
use crate::lang::lr::BuildItemSets;

pub struct LR0ABuilder<'a> {
    grammar: &'a Grammar,
}

impl BuildItemSets<LR0Item> for LR0ABuilder<'_> {
    fn start_item(&self) -> LR0Item {
        LR0Item::new(self.grammar.production_count() - 1, 0)
    }

    fn advance(&self, item: &LR0Item) -> LR0Item {
        LR0Item::new(item.production, item.pos + 1)
    }

    fn symbol_at_dot(&self, item: &LR0Item) -> Option<Symbol> {
        item.symbol_at_dot(self.grammar)
    }

    fn closure(&self, old_items: &BTreeSet<LR0Item>) -> BTreeSet<LR0Item> {
        let mut items     = old_items.clone();
        let mut new_items = old_items.clone();
        
        let mut done = false;
        
        while !done {
            done = true;

            for item in &items {
                if let Some(Symbol::Variable(A)) = item.symbol_at_dot(self.grammar) {
                    for alt in self.grammar.rule(A).alt_indices() {
                        if new_items.insert(LR0Item::new(alt, 0)) {
                            done = false;
                        }
                    }
                }
            }

            items = new_items.clone();
        }
    
        items
    }
}

impl<'a> LR0ABuilder<'a> {
    #[must_use]
    pub fn new(grammar: &'a Grammar) -> Self {
        LR0ABuilder {
            grammar,
        }
    }

    #[must_use]
    pub fn build(self) -> LR0A {
        let (itemsets, gotos) = <Self as BuildItemSets<LR0Item>>::build(&self);

        LR0A {
            states: itemsets.into_iter()
                .zip(gotos)
                .map(|(items, next)| State { items: items.into_iter().collect(), next })
                .collect()
        }
    }
}