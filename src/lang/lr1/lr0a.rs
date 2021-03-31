#![allow(non_snake_case)]

use std::collections::{BTreeSet, HashMap};
use crate::lang::cfg::{Grammar, Symbol};
use crate::debug::StringBuilder;
use super::LR0Item;

pub struct LR0A {
    pub(super) states: Vec<State>,
}

pub struct State {
    pub items: BTreeSet<LR0Item>,
    pub next: HashMap<Symbol, usize>,
}

impl LR0A {
    #[must_use]
    pub fn states(&self) -> &[State] {
        &self.states
    }

    #[must_use]
    pub fn dot<T, U>(&self, grammar: &Grammar, word_names: &[T], var_names: &[U], print_itemsets: bool) -> String
    where
        T: std::fmt::Display,
        U: std::fmt::Display,
    {
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

fn format_item<F, G, T, U>(grammar: &Grammar, var: usize, item: &LR0Item, word_labelling: F, var_labelling: G) -> String
    where F: Fn(usize) -> T,
          G: Fn(usize) -> U,
          T: std::fmt::Display,
          U: std::fmt::Display,
{
    let alt = &grammar.alt(item.alt);

    format!("[{} &rarr; {}&bull;{}]", 
        var_labelling(var),
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
    )
}

#[must_use]
fn dot_with_labelling_internal<F, G, T, U>(grammar: &Grammar, lr0a: &LR0A, word_labelling: F, var_labelling: G, print_itemsets: bool) -> String
    where F: Fn(usize) -> T + Copy,
          G: Fn(usize) -> U + Copy,
          T: std::fmt::Display,
          U: std::fmt::Display,
{
    let alt2var: Vec<_> = grammar.rules().enumerate().flat_map(|(i, rule)| rule.alts().map(|_| i).collect::<Vec<_>>()).collect();

    let mut dot = StringBuilder::new();

    dot.writeln("digraph CC {");
    dot.indent();
    dot.writeln("rankdir=LR;");

    dot.newline();

    if print_itemsets {
        dot.writeln("node[shape=plain];");
        for (id, state) in lr0a.states.iter().enumerate() {
            dot.writeln(&format!("s{}[label=", id));
            dot.indent();
            dot.writeln("<<table border=\"1\" cellborder=\"0\">");
            dot.indent();
            dot.writeln(&format!("<tr><td align=\"center\"><b>s{}</b></td></tr>", id));
            for item in &state.items {
                dot.writeln(&format!("<tr><td align=\"left\">{}</td></tr>", format_item(grammar, alt2var[item.alt], item, word_labelling, var_labelling)));
            }
            dot.unindent();
            dot.writeln("</table>>];");
            dot.unindent();
        }
    } else {
        dot.writeln("node[shape=rectangle];");
        for (id, _) in lr0a.states.iter().enumerate() {
            dot.writeln(&format!("s{};", id));
        }
    }

    dot.newline();

    for (A, state) in lr0a.states.iter().enumerate() {
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