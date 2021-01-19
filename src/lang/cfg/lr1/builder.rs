use std::collections::{HashMap, BTreeSet};
use std::collections::VecDeque;
use std::iter::once;
use std::rc::Rc;

use super::{DFA, State, Item};
use super::{Grammar, Symbol};
use super::super::first::First;

type ItemSet = BTreeSet<Item>;

pub struct DFABuilder<'a> {
    grammar: &'a Grammar,
    first: First,
}

impl<'a> DFABuilder<'a> {
    #[must_use]
    pub fn new(grammar: &'a Grammar) -> Self {
        DFABuilder {
            grammar,
            first: First::new(grammar),
        }
    }

    #[must_use]
    pub fn build(self) -> DFA {   
        // Last rule the start
        let initial_items = Rc::new(
            self.closure(
                &once(Item {
                    rule: self.grammar.rule_count() - 1,
                    alt: self.grammar.alt_count() - 1,
                    pos: 0,
                    successor: None
                }).collect()
            )
        );

        let mut itemsets = vec![initial_items.clone()];
        let mut gotos: Vec<HashMap<Symbol, usize>> = vec![HashMap::new()];

         // Item sets we've seen so far.
        let mut table: HashMap<_, usize> = once((initial_items.clone(), 0)).collect();
        // NOTE: A stack could be used here instead; but by using a queue,
        //       the iteration step of the outer-most loop will correspond
        //       to the index of the item set in CC we are currently
        //       transitioning from.
        let mut queue: VecDeque<_> = once(initial_items).collect();

        let mut i = 0_usize;

        while let Some(item_set) = queue.pop_front() {
            let mut iter1 = item_set.iter();
            let mut iter2 = iter1.clone();

            while let Some(item) = iter1.next() {
                if let Some(x) = self.grammar.symbol_at_dot(item) {
                    // x has already been processed
                    if gotos[i].contains_key(&x) {
                        continue;
                    }
                    
                    // Previously processed items in item_set (those before
                    // iter2) are guaranteed to not contribute to the output
                    // item set. As such, goto is only required to process
                    // from iter2 onwards.
                    let temp = self.goto(iter2, &x);
                    
                    let j = if let Some(&index) = table.get(&temp) {
                        index
                    } else {
                        // If temp hasn't been seen yet, add to CC and
                        // mark to be processed.
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

        DFA {
            states: itemsets.into_iter()
                .map(Rc::try_unwrap)
                .map(Result::unwrap)
                .zip(gotos)
                .map(|(items, next)| State { items, next })
                .collect()
        }
    }

    fn closure(&self, old_items: &ItemSet) -> ItemSet {
        let mut items     = old_items.clone();
        let mut new_items = old_items.clone();
        
        let mut done = false;
        
        while !done {
            done = true;

            for item in &items {
                if let Some(Symbol::Variable(rule)) = self.grammar.symbol_at_dot(item) {
                    match self.grammar.symbol_after_dot(item) {
                        None => {
                            for alt in self.grammar.rule(rule).alt_indices() {
                                if new_items.insert(Item { rule, alt, pos: 0, successor: item.successor }) {
                                    done = false;
                                }
                            }
                        },
                        Some(Symbol::Terminal(a)) => {
                            for alt in self.grammar.rule(rule).alt_indices() {
                                if new_items.insert(Item { rule, alt, pos: 0, successor: Some(a) }) {
                                    done = false;
                                }
                            }
                        },
                        Some(Symbol::Variable(A)) => {
                            let first_A = self.first.get(A);

                            // NOTE: if first contains epsilon, it is guaranteed to be at index 0
                            if first_A[0].is_some() {
                                for &successor in first_A {
                                    for alt in self.grammar.rule(rule).alt_indices() {
                                        if new_items.insert(Item { rule, alt, pos: 0, successor }) {
                                            done = false;
                                        }
                                    }
                                }
                            } else {
                                for &successor in first_A[1..].iter().chain(once(&item.successor)) {
                                    for alt in self.grammar.rule(rule).alt_indices() {
                                        if new_items.insert(Item { rule, alt, pos: 0, successor }) {
                                            done = false;
                                        }
                                    }
                                }
                            }
                        },
                    };
                }
            }

            items = new_items.clone();
        }
    
        items
    }

    fn goto<'b, I: Iterator<Item=&'b Item>>(&self, items: I, x: &Symbol) -> ItemSet {
        self.closure(&items.filter_map(|item| {
            if let Some(y) = self.grammar.symbol_at_dot(item) {
                if *x == y {
                    return Some(Item { pos: item.pos + 1, ..*item });
                }
            }
            None
        }).collect::<ItemSet>())
    }
}