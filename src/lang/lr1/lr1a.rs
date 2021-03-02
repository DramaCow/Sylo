#![allow(non_snake_case)]

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::iter::once;
use std::rc::Rc;
use crate::lang::cfg::{Grammar, Symbol, First};
use crate::debug::StringBuilder;
use super::Item;

pub struct LR1A {
    states: Vec<State>,
}

pub struct State {
    pub items: BTreeSet<Item>,
    pub next: HashMap<Symbol, usize>,
}

impl From<&Grammar> for LR1A {
    fn from(grammar: &Grammar) -> Self {
        LR1ABuilder::new(grammar).build()
    }
}

impl LR1A {
    #[must_use]
    pub fn states(&self) -> &[State] {
        &self.states
    }

    #[must_use]
    pub fn dot<T: std::fmt::Display, U: std::fmt::Display>(&self, grammar: &Grammar, word_names: &[T], var_names: &[U], print_itemsets: bool) -> String {
        let word_to_string = |word: usize| {
            format!("{}", word_names[word])
        };

        let var_to_string = |var: usize| {
            if var < var_names.len() {
                format!("{}", var_names[var])
            } else {
                "***START***".to_string()
            }
        };

        dot_with_labelling_internal(grammar, self, word_to_string, var_to_string, print_itemsets)
    }
}

// =================
// === INTERNALS ===
// =================

type ItemSet = BTreeSet<Item>;

struct LR1ABuilder<'a> {
    grammar: &'a Grammar,
    first: First,
}

impl<'a> LR1ABuilder<'a> {
    fn new(grammar: &'a Grammar) -> Self {
        LR1ABuilder {
            grammar,
            first: First::new(grammar),
        }
    }

    fn build(self) -> LR1A {   
        let initial_items = Rc::new(
            self.closure(
                // NOTE: the last rule in the grammar is the implicit start
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

        LR1A {
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
                if let Some(Symbol::Variable(rule)) = item.symbol_at_dot(&self.grammar) {
                    match item.symbol_after_dot(&self.grammar) {
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
            if let Some(y) = item.symbol_at_dot(&self.grammar) {
                if *x == y {
                    return Some(Item { pos: item.pos + 1, ..*item });
                }
            }
            None
        }).collect::<ItemSet>())
    }
}

fn format_item<F, G, T, U>(grammar: &Grammar, item: &Item, word_labelling: F, var_labelling: G) -> String
    where F: Fn(usize) -> T,
          G: Fn(usize) -> U,
          T: std::fmt::Display,
          U: std::fmt::Display,
{
    let alt = &grammar.alt(item.alt);

    format!("[{} &rarr; {}&bull;{}, {}]", 
        var_labelling(item.rule),
        if item.pos == 0 { "".to_string() } else { 
            alt[..item.pos].iter().map(|symbol| match symbol {
                Symbol::Terminal(a) => format!("{}", word_labelling(*a)),
                Symbol::Variable(A) => format!("{}", var_labelling(*A)),
            }).collect::<Vec<_>>().join(" ")
        },
        if item.pos >= alt.len() { "".to_string() } else { 
            alt[item.pos..].iter().map(|symbol| match symbol {
                Symbol::Terminal(a) => format!("{}", word_labelling(*a)),
                Symbol::Variable(A) => format!("{}", var_labelling(*A)),
            }).collect::<Vec<_>>().join(" ")
        },
        item.successor.map_or("$".to_string(), |a| format!("{}", word_labelling(a)))
    )
}

#[must_use]
fn dot_with_labelling_internal<F, G, T, U>(grammar: &Grammar, lr1a: &LR1A, word_labelling: F, var_labelling: G, print_itemsets: bool) -> String
    where F: Fn(usize) -> T + Copy,
          G: Fn(usize) -> U + Copy,
          T: std::fmt::Display,
          U: std::fmt::Display,
{
    let mut dot = StringBuilder::new();

    dot.writeln("digraph CC {");
    dot.indent();
    dot.writeln("rankdir=LR;");

    dot.newline();

    if print_itemsets {
        dot.writeln("node[shape=plain];");
        for (id, state) in lr1a.states.iter().enumerate() {
            dot.writeln(&format!("s{}[label=", id));
            dot.indent();
            dot.writeln("<<table border=\"1\" cellborder=\"0\">");
            dot.indent();
            dot.writeln(&format!("<tr><td align=\"center\"><b>s{}</b></td></tr>", id));
            for item in &state.items {
                dot.writeln(&format!("<tr><td align=\"left\">{}</td></tr>", format_item(grammar, item, word_labelling, var_labelling)));
            }
            dot.unindent();
            dot.writeln("</table>>];");
            dot.unindent();
        }
    } else {
        dot.writeln("node[shape=rectangle];");
        for (id, _) in lr1a.states.iter().enumerate() {
            dot.writeln(&format!("s{};", id));
        }
    }

    dot.newline();

    for (A, state) in lr1a.states.iter().enumerate() {
        for (symbol, B) in &state.next {
            dot.writeln(&format!("s{}->s{}[label={:?}];", A, B, 
                match symbol {
                    Symbol::Terminal(a) => format!("{}", word_labelling(*a)),
                    Symbol::Variable(A) => format!("{}", var_labelling(*A)),
                }
            ));
        }
    }

    dot.unindent();
    dot.writeln("}");

    dot.build()
}