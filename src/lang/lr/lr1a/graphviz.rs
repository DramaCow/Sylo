use crate::lang::cfg::{Grammar, Symbol};
use crate::utils::IndentWriter;
use super::{LR1A, LR1Item};

/// # Errors
pub fn dot_with_labelling<F, G, T, U>(grammar: &Grammar, lr1a: &LR1A, word_labelling: F, var_labelling: G, print_itemsets: bool) -> Result<String, std::fmt::Error>
    where F: Fn(usize) -> T + Copy,
          G: Fn(usize) -> U + Copy,
          T: std::fmt::Display,
          U: std::fmt::Display,
{
    let alt2var: Vec<_> = grammar.rules().enumerate().flat_map(|(i, rule)| rule.alts().map(move |_| i)).collect();

    let mut fmt = IndentWriter::new(String::new());

    writeln!(fmt, "digraph CC {{")?;
    fmt.indent();
    writeln!(fmt, "rankdir=LR;")?;

    writeln!(fmt)?;

    if print_itemsets {
        writeln!(fmt, "node[shape=plain];")?;
        for (id, state) in lr1a.states.iter().enumerate() {
            writeln!(fmt, "s{}[label=", id)?;
            fmt.indent();
            writeln!(fmt, "<<table border=\"1\" cellborder=\"0\">")?;
            fmt.indent();
            writeln!(fmt, "<tr><td align=\"center\"><b>s{}</b></td></tr>", id)?;
            for item in &state.items {
                writeln!(fmt, "<tr><td align=\"left\">{}</td></tr>", format_item(grammar, alt2var[item.lr0_item.production], item, word_labelling, var_labelling))?;
            }
            fmt.unindent();
            writeln!(fmt, "</table>>];")?;
            fmt.unindent();
        }
    } else {
        writeln!(fmt, "node[shape=rectangle];")?;
        for (id, _) in lr1a.states.iter().enumerate() {
            writeln!(fmt, "s{};", id)?;
        }
    }

    writeln!(fmt)?;

    for (A, state) in lr1a.states.iter().enumerate() {
        for (symbol, B) in &state.next {
            writeln!(fmt, "s{}->s{}[label={:?}];", A, B, 
                match symbol {
                    Symbol::Terminal(a) => format!("{}", word_labelling(*a)),
                    Symbol::Variable(A) => format!("{}", var_labelling(*A)),
                }
            )?;
        }
    }

    fmt.unindent();
    writeln!(fmt, "}}")?;

    Ok(fmt.build())
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
    let alt = &grammar.alt(item.lr0_item.production);

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