use std::collections::{BTreeSet, HashMap};
use std::fmt;
use crate::debug::StringBuilder;

use super::{Grammar, Symbol};

/// LR(1) item.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Item {
    pub rule: usize,              // index of rule
    pub alt:  usize,              // index of alt
    pub pos:  usize,              // index of dot in alt
    // If all of alt is present in context (LHS), and the suceeding
    // symbol is successor, then lhs can be reduced to rule (RHS).
    pub successor: Option<usize>, // class of successor terminal
}

pub struct DFA {
    pub(crate) states: Vec<State>,
}

pub struct State {
    pub items: BTreeSet<Item>,
    pub next: HashMap<Symbol, usize>,
}

impl From<&Grammar> for DFA {
    fn from(grammar: &Grammar) -> Self {
        DFABuilder::new(grammar).build()
    }
}

impl DFA {
    #[must_use]
    pub fn dot<T: std::fmt::Display, U: std::fmt::Display>(&self, grammar: &Grammar, word_names: &[T], var_names: &[U]) -> String {
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

        dot_with_labelling_internal(grammar, self, word_to_string, var_to_string)
    }
}

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if let Some(delta) = self.successor {
            f.write_str(&format!("(alt: {}, pos: {}) --> {}", self.alt, self.pos, delta))
        } else {
            f.write_str(&format!("(alt: {}, pos: {}) --> \u{03b5}", self.alt, self.pos))
        }
    }
}

// =================
// === INTERNALS ===
// =================

impl Grammar {
    pub(crate) fn is_complete(&self, item: &Item) -> bool {
        item.pos >= self.alt(item.alt).len()
    }

    pub(crate) fn symbol_at_dot(&self, item: &Item) -> Option<Symbol> {
        let alt = &self.alt(item.alt);
        if item.pos < alt.len() {
            Some(alt[item.pos])
        } else {
            None
        }
    }

    pub(crate) fn symbol_after_dot(&self, item: &Item) -> Option<Symbol> {
        let alt = &self.alt(item.alt);
        if item.pos + 1 < alt.len() {
            Some(alt[item.pos+1])
        } else {
            None
        }
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
fn dot_with_labelling_internal<F, G, T, U>(grammar: &Grammar, dfa: &DFA, word_labelling: F, var_labelling: G) -> String
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

    dot.writeln("node[shape=plain];");
    for (id, state) in dfa.states.iter().enumerate() {
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

    dot.newline();

    for (A, state) in dfa.states.iter().enumerate() {
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

mod builder;
use builder::DFABuilder;