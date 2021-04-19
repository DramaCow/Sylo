#![allow(non_snake_case)]

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::iter::once;
use std::rc::Rc;
use crate::lang::cfg::{Grammar, Symbol};
use super::{LR0Item, LR0A, lr0a::State};

pub struct LR0ABuilder<'a> {
    grammar: &'a Grammar,
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
        let initial_items = Rc::new(
            self.closure(
                // NOTE: the last rule in the grammar is the implicit start
                &once(LR0Item::new(self.grammar.alt_count() - 1, 0)).collect()
            )
        );

        let mut itemsets = vec![initial_items.clone()];
        let mut gotos: Vec<HashMap<Symbol, usize>> = vec![HashMap::new()];

        // Item sets we've seen so far mapped to indices in itemsets vector.
        let mut table: HashMap<_, usize> = once((initial_items.clone(), 0)).collect();

        // Queue of itemsets to process.
        // NOTE: A stack could be used here instead; but by using a queue,
        //       the iteration step of the outer-most loop (i) will correspond
        //       to the index of the item set in CC we are currently
        //       transitioning from.
        let mut queue: VecDeque<_> = once(initial_items).collect();

        let mut i = 0_usize;

        while let Some(item_set) = queue.pop_front() {
            let mut iter1 = item_set.iter();
            let mut iter2 = iter1.clone();

            while let Some(item) = iter1.next() {
                if let Some(x) = item.symbol_at_dot(&self.grammar) {
                    // x has already been processed
                    if gotos[i].contains_key(&x) {
                        continue;
                    }
                    
                    // NOTE: Previously processed items in item_set (those before
                    //       iter2) are guaranteed to not contribute to the output
                    //       item set. As such, goto is only required to process
                    //       from iter2 onwards.
                    let temp = self.goto(iter2, &x);
                    
                    // Check if temp is already in itemsets. If not, we
                    // add to itemsets and push on to process queue.
                    let j = if let Some(&index) = table.get(&temp) {
                        index
                    } else {
                        let new_index = itemsets.len();
                        let temp_rc = Rc::new(temp);

                        itemsets.push(temp_rc.clone());
                        gotos.push(HashMap::new());

                        table.insert(temp_rc.clone(), new_index);
                        queue.push_back(temp_rc);

                        new_index
                    };

                    // Record transition on x
                    gotos[i].insert(x, j);

                    iter2 = iter1.clone();
                }
            }

            i += 1;
        }

        // forces out-of-scope early so all
        // reference counts get decremented.
        drop(table);

        LR0A {
            states: itemsets.into_iter()
                .map(Rc::try_unwrap)
                .map(Result::unwrap)
                .zip(gotos)
                .map(|(items, next)| State { items, next })
                .collect()
        }
    }
}

// =================
// === INTERNALS ===
// =================

type ItemSet = BTreeSet<LR0Item>;

impl LR0ABuilder<'_> {
    fn closure(&self, old_items: &ItemSet) -> ItemSet {
        let mut items     = old_items.clone();
        let mut new_items = old_items.clone();
        
        let mut done = false;
        
        while !done {
            done = true;

            for item in &items {
                if let Some(Symbol::Variable(var)) = item.symbol_at_dot(&self.grammar) {
                    for alt in self.grammar.rule(var).alt_indices() {
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

    fn goto<'a, I: Iterator<Item=&'a LR0Item>>(&self, items: I, x: &Symbol) -> ItemSet {
        self.closure(&items.filter_map(|item| {
            if let Some(y) = item.symbol_at_dot(&self.grammar) {
                if *x == y {
                    return Some(LR0Item::new(item.alt, item.pos + 1));
                }
            }
            None
        }).collect::<ItemSet>())
    }
}