#![allow(non_snake_case)]

use std::collections::{BTreeSet, HashMap};
use crate::lang::cfg::{Grammar, Symbol};
use crate::debug::StringBuilder;
use super::LR1Item;

pub struct LR1A {
    pub(super) states: Vec<State>,
}

pub struct State {
    pub items: BTreeSet<LR1Item>,
    pub next: HashMap<Symbol, usize>,
}

impl LR1A {
    #[must_use]
    pub fn states(&self) -> &[State] {
        &self.states
    }

    /// # Errors
    pub fn dot<T, U>(&self, grammar: &Grammar, word_names: &[T], var_names: &[U], print_itemsets: bool) -> Result<String, std::fmt::Error>
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
                "S'".to_string()
            }
        };

        dot_with_labelling_internal(grammar, self, word_to_string, var_to_string, print_itemsets)
    }
}

// =================
// === INTERNALS ===
// =================

fn format_item<F, G, T, U>(grammar: &Grammar, var: usize, item: &LR1Item, word_labelling: F, var_labelling: G) -> String
    where F: Fn(usize) -> T,
          G: Fn(usize) -> U,
          T: std::fmt::Display,
          U: std::fmt::Display,
{
    let alt = &grammar.alt(item.lr0_item.alt);

    format!("[{} &rarr; {}&bull;{}, {}]", 
        var_labelling(var),
        if item.lr0_item.pos == 0 { "".to_string() } else { 
            alt[..item.lr0_item.pos].iter().map(|symbol| match symbol {
                Symbol::Terminal(a) => format!("{}", word_labelling(*a)),
                Symbol::Variable(A) => format!("{}", var_labelling(*A)),
            }).collect::<Vec<_>>().join(" ")
        },
        if item.lr0_item.pos >= alt.len() { "".to_string() } else { 
            alt[item.lr0_item.pos..].iter().map(|symbol| match symbol {
                Symbol::Terminal(a) => format!("{}", word_labelling(*a)),
                Symbol::Variable(A) => format!("{}", var_labelling(*A)),
            }).collect::<Vec<_>>().join(" ")
        },
        item.lookahead.map_or("$".to_string(), |a| format!("{}", word_labelling(a)))
    )
}

/// # Errors
fn dot_with_labelling_internal<F, G, T, U>(grammar: &Grammar, lr1a: &LR1A, word_labelling: F, var_labelling: G, print_itemsets: bool) -> Result<String, std::fmt::Error>
    where F: Fn(usize) -> T + Copy,
          G: Fn(usize) -> U + Copy,
          T: std::fmt::Display,
          U: std::fmt::Display,
{
    let alt2var: Vec<_> = grammar.rules().enumerate().flat_map(|(i, rule)| rule.alts().map(move |_| i)).collect();

    let mut dot = StringBuilder::new();

    writeln!(dot, "digraph CC {{")?;
    dot.indent();
    writeln!(dot, "rankdir=LR;")?;

    dot.newline();

    if print_itemsets {
        writeln!(dot, "node[shape=plain];")?;
        for (id, state) in lr1a.states.iter().enumerate() {
            writeln!(dot, "s{}[label=", id)?;
            dot.indent();
            writeln!(dot, "<<table border=\"1\" cellborder=\"0\">")?;
            dot.indent();
            writeln!(dot, "<tr><td align=\"center\"><b>s{}</b></td></tr>", id)?;
            for item in &state.items {
                writeln!(dot, "<tr><td align=\"left\">{}</td></tr>", format_item(grammar, alt2var[item.lr0_item.alt], item, word_labelling, var_labelling))?;
            }
            dot.unindent();
            writeln!(dot, "</table>>];")?;
            dot.unindent();
        }
    } else {
        writeln!(dot, "node[shape=rectangle];")?;
        for (id, _) in lr1a.states.iter().enumerate() {
            writeln!(dot, "s{};", id)?;
        }
    }

    dot.newline();

    for (A, state) in lr1a.states.iter().enumerate() {
        for (symbol, B) in &state.next {
            writeln!(dot, "s{}->s{}[label={:?}]?;", A, B, 
                match symbol {
                    Symbol::Terminal(a) => format!("{}", word_labelling(*a)),
                    Symbol::Variable(A) => format!("{}", var_labelling(*A)),
                }
            )?;
        }
    }

    dot.unindent();
    writeln!(dot, "}}")?;

    Ok(dot.build())
}