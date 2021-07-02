use crate::lang::cfg::{Grammar, Symbol};
use crate::utils::IndentWriter;
use super::{LR0A, State, LR0Item};
use std::fmt::Write;
use std::cmp::Ordering::{Less, Greater};

pub struct LR0ADotWriter<'a, W, F, T>
where
    W: Write,
    F: Fn(Symbol) -> T,
    T: std::fmt::Display,
{
    fmt: IndentWriter<W>,
    lr0a: &'a LR0A,
    grammar: &'a Grammar,
    labelling: F,
    alt2var: Vec<usize>,
}

impl<'a, W, F, T> LR0ADotWriter<'a, W, F, T>
where
    W: Write,
    F: Fn(Symbol) -> T,
    T: std::fmt::Display,
{
    #[must_use]
    pub fn new(fmt: W, lr0a: &'a LR0A, grammar: &'a Grammar, labelling: F) -> Self {
        let alt2var: Vec<_> = grammar.rules().enumerate().flat_map(|(i, rule)| rule.alts().map(move |_| i)).collect();

        Self {
            fmt: IndentWriter::new(fmt),
            lr0a,
            grammar,
            labelling,
            alt2var,
        }
    }

    /// # Errors
    pub fn build(mut self) -> Result<W, std::fmt::Error> {
        writeln!(self.fmt, "digraph CC {{")?;
        self.fmt.indent();
        writeln!(self.fmt, "rankdir=LR;")?;

        writeln!(self.fmt);

        writeln!(self.fmt, "node[shape=point]; q;")?;
        writeln!(self.fmt, "node[shape=plain]; accept[label=<<b>ACCEPT</b>>];");
        for (id, state) in self.lr0a.states.iter().enumerate() {
            self.format_state(id, state)?;
        }

        writeln!(self.fmt);

        writeln!(self.fmt, "q->s0;")?;
        writeln!(self.fmt, "s{}->accept[label=\"&#9633;\"];", *self.lr0a.states[0].next.get(&Symbol::Variable(0)).unwrap());
        for (A, state) in self.lr0a.states.iter().enumerate() {
            for (symbol, B) in &state.next {
                writeln!(self.fmt, "s{}->s{}[label=\"{}\"];", A, B, (self.labelling)(*symbol))?;
            }
        }

        self.fmt.unindent();
        writeln!(self.fmt, "}}")?;

        Ok(self.fmt.build())
    }

    fn format_state(&mut self, id: usize, state: &State) -> Result<(), std::fmt::Error> {
        let items = {
            let mut items: Vec<_> = state.items.iter().copied().collect();
            items.sort_by(|a, b| {
                match (a.is_kernel_item(self.grammar), b.is_kernel_item(self.grammar)) {
                    (false, false) | (true, true) => {
                        let c1 = self.alt2var[a.production] == self.grammar.var_count() - 1;
                        let c2 = self.alt2var[b.production] == self.grammar.var_count() - 1;
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
            items
        };

        writeln!(self.fmt, "s{}[label=", id)?;
        self.fmt.indent();
        writeln!(self.fmt, "<<table border=\"1\" cellborder=\"0\">")?;
        self.fmt.indent();
        writeln!(self.fmt, "<tr><td align=\"center\"><b>s{}</b></td></tr>", id)?;
        for item in &items {
            if item.is_kernel_item(self.grammar) {
                writeln!(self.fmt, "<tr><td align=\"left\">{}</td></tr>", self.format_item(item))?;   
            } else {
                writeln!(self.fmt, "<tr><td bgcolor=\"grey\" align=\"left\">{}</td></tr>", self.format_item(item))?;
            }
        }
        self.fmt.unindent();
        writeln!(self.fmt, "</table>>];")?;
        self.fmt.unindent();
        Ok(())
    }

    fn format_item(&self, item: &LR0Item) -> String {
        let alt = self.grammar.alt(item.production);

        format!("{} &rarr; {}&bull;{}",
            (self.labelling)(Symbol::Variable(self.alt2var[item.production])),
            alt[..item.pos].iter().map(|&symbol| (self.labelling)(symbol).to_string()).collect::<Vec<_>>().join(" "),
            alt[item.pos..].iter().map(|&symbol| (self.labelling)(symbol).to_string()).collect::<Vec<_>>().join(" "),
        )
    }
}