/// TODO: Should not be a child of lr0a module.

use crate::lang::cfg::{Grammar, Symbol};
use crate::utils::IndentWriter;
use super::{LR0A, State, LR0Item};
use std::fmt::Write;
use std::cmp::Ordering::{Less, Greater};

/// # Errors
pub fn dot_with_labelling<F, T>(grammar: &Grammar, lr0a: &LR0A, labelling: F) -> Result<String, std::fmt::Error>
    where F: Fn(Symbol) -> T + Copy,
          T: std::fmt::Display,
{
    let alt2var: Vec<_> = grammar.rules().enumerate().flat_map(|(i, rule)| rule.alts().map(move |_| i)).collect();
    let mut fmt = IndentWriter::new(String::new());
    
    writeln!(fmt, "digraph CC {{")?;
    fmt.indent();
    writeln!(fmt, "rankdir=LR;")?;

    writeln!(fmt);

    writeln!(fmt, "node[shape=point]; q;")?;
    writeln!(fmt, "node[shape=plain]; accept[label=<<b>ACCEPT</b>>];");
    for (id, state) in lr0a.states.iter().enumerate() {
        format_state(&mut fmt, &grammar, id, state, &alt2var, labelling)?;
    }

    writeln!(fmt);
    
    writeln!(fmt, "q->s0;")?;
    writeln!(fmt, "s{}->accept[label=\"&#9633;\"];", *lr0a.states[0].next.get(&Symbol::Variable(0)).unwrap());
    for (A, state) in lr0a.states.iter().enumerate() {
        for (symbol, B) in &state.next {
            writeln!(fmt, "s{}->s{}[label=\"{}\"];", A, B, labelling(*symbol))?;
        }
    }

    fmt.unindent();
    writeln!(fmt, "}}")?;

    Ok(fmt.build())
}

// =================
// === INTERNALS ===
// =================

fn format_item<F, T>(grammar: &Grammar, var: usize, item: &LR0Item, labelling: F) -> String
    where F: Fn(Symbol) -> T,
          T: std::fmt::Display,
{
    let alt = &grammar.alt(item.production);

    format!("{} &rarr; {}&bull;{}",
        labelling(Symbol::Variable(var)),
        alt[..item.pos].iter().map(|&symbol| labelling(symbol).to_string()).collect::<Vec<_>>().join(" "),
        alt[item.pos..].iter().map(|&symbol| labelling(symbol).to_string()).collect::<Vec<_>>().join(" "),
    )
}

fn format_state<W, F, T>(fmt: &mut IndentWriter<W>, grammar: &Grammar, id: usize, state: &State, alt2var: &[usize], labelling: F) -> Result<(), std::fmt::Error>
    where W: Write,
          F: Fn(Symbol) -> T + Copy,
          T: std::fmt::Display,
{
    let mut items: Vec<_> = state.items.iter().copied().collect();
    items.sort_by(|a, b| {
        match (a.is_kernel_item(&grammar), b.is_kernel_item(&grammar)) {
            (false, false) | (true, true) => {
                let c1 = alt2var[a.production] == grammar.var_count() - 1;
                let c2 = alt2var[b.production] == grammar.var_count() - 1;
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

    writeln!(fmt, "s{}[label=", id)?;
    fmt.indent();
    writeln!(fmt, "<<table border=\"1\" cellborder=\"0\">")?;
    fmt.indent();
    writeln!(fmt, "<tr><td align=\"center\"><b>s{}</b></td></tr>", id)?;
    for item in &items {
        if item.is_kernel_item(&grammar) {
            writeln!(fmt, "<tr><td align=\"left\">{}</td></tr>", format_item(grammar, alt2var[item.production], item, labelling))?;   
        } else {
            writeln!(fmt, "<tr><td bgcolor=\"grey\" align=\"left\">{}</td></tr>", format_item(grammar, alt2var[item.production], item, labelling))?;
        }
    }
    fmt.unindent();
    writeln!(fmt, "</table>>];")?;
    fmt.unindent();
    Ok(())
}