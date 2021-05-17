#![allow(non_snake_case)]

use std::collections::{BTreeSet, HashMap};
use crate::lang::cfg::{Grammar, Symbol};
use crate::utils::StringBuilder;
use super::LR0Item;
use std::cmp::Ordering::{Less, Greater};

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

fn format_item<F, G, T, U>(grammar: &Grammar, var: usize, item: &LR0Item, word_labelling: F, var_labelling: G) -> String
    where F: Fn(usize) -> T,
          G: Fn(usize) -> U,
          T: std::fmt::Display,
          U: std::fmt::Display,
{
    let alt = &grammar.alt(item.alt);

    format!("{} &rarr; {}&bull;{}",
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

fn format_state<F, G, T, U>(fmt: &mut StringBuilder, grammar: &Grammar, id: usize, state: &State, alt2var: &[usize], word_labelling: F, var_labelling: G) -> Result<(), std::fmt::Error>
    where F: Fn(usize) -> T + Copy,
          G: Fn(usize) -> U + Copy,
          T: std::fmt::Display,
          U: std::fmt::Display,
{
    writeln!(fmt, "s{}[label=", id)?;
    fmt.indent();
    writeln!(fmt, "<<table border=\"1\" cellborder=\"0\">")?;
    fmt.indent();
    writeln!(fmt, "<tr><td align=\"center\"><b>s{}</b></td></tr>", id)?;

    let mut items: Vec<_> = state.items.iter().copied().collect();
    items.sort_by(|a, b| {
        match (a.is_kernel_item(&grammar), b.is_kernel_item(&grammar)) {
            (false, false) | (true, true) => {
                let c1 = alt2var[a.alt] == grammar.var_count() - 1;
                let c2 = alt2var[b.alt] == grammar.var_count() - 1;
                match (c1, c2) {
                    (false, false) | (true, true) => a.cmp(&b),
                    (false, true) => Greater,
                    (true, false) => Less,
                }
            }
            (false, true) => Greater,
            (true, false) => Less,
        }
    });
    for item in &items {
        if item.is_kernel_item(&grammar) {
            writeln!(fmt, "<tr><td align=\"left\">{}</td></tr>", format_item(grammar, alt2var[item.alt], item, word_labelling, var_labelling))?;   
        } else {
            writeln!(fmt, "<tr><td bgcolor=\"grey\" align=\"left\">{}</td></tr>", format_item(grammar, alt2var[item.alt], item, word_labelling, var_labelling))?;
        }
    }
    fmt.unindent();
    writeln!(fmt, "</table>>];")?;
    fmt.unindent();
    Ok(())
}

/// # Errors
fn dot_with_labelling_internal<F, G, T, U>(grammar: &Grammar, lr0a: &LR0A, word_labelling: F, var_labelling: G, print_itemsets: bool) -> Result<String, std::fmt::Error>
    where F: Fn(usize) -> T + Copy,
          G: Fn(usize) -> U + Copy,
          T: std::fmt::Display,
          U: std::fmt::Display,
{
    let mut dot = StringBuilder::new();
    
    writeln!(dot, "digraph CC {{")?;
    dot.indent();
    writeln!(dot, "rankdir=LR;")?;
    
    dot.newline();
    
    writeln!(dot, "node[shape=point]; q;")?;
    if print_itemsets {
        let alt2var: Vec<_> = grammar.rules().enumerate().flat_map(|(i, rule)| rule.alts().map(move |_| i)).collect();
        let is_complete: Vec<_> = lr0a.states.iter().map(|state| {
            for item in &state.items {
                if item.is_complete(&grammar) && alt2var[item.alt] == grammar.var_count() - 1 {
                    return true;
                }
            }
            false
        }).collect();
        writeln!(dot, "node[shape=box margin=0.05];")?;
        for (id, state) in lr0a.states.iter().enumerate().filter(|(id, _)| is_complete[*id]) {
            format_state(&mut dot, &grammar, id, state, &alt2var, word_labelling, var_labelling)?;
        }
        writeln!(dot, "node[shape=plain];")?;
        for (id, state) in lr0a.states.iter().enumerate().filter(|(id, _)| !is_complete[*id]) {
            format_state(&mut dot, &grammar, id, state, &alt2var, word_labelling, var_labelling)?;
        }
    } else {
        writeln!(dot, "node[shape=rectangle];")?;
        for (id, _) in lr0a.states.iter().enumerate() {
            writeln!(dot, "s{};", id)?;
        }
    }

    dot.newline();

    writeln!(dot, "q->s0;")?;
    for (A, state) in lr0a.states.iter().enumerate() {
        for (symbol, B) in &state.next {
            writeln!(dot, "s{}->s{}[label={:?}];", A, B, 
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