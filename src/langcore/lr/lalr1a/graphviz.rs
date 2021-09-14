use crate::langcore::cfg::{Grammar, Symbol};
use crate::utils::IndentWriter;
use super::{LALR1A, State, LR0Item, StateReductionPair};
use std::fmt::Write;
use std::cmp::Ordering::{Less, Greater};

pub struct LALR1ADotWriter<'a, W, F, T>
where
    W: Write,
    F: Fn(Symbol) -> T,
    T: std::fmt::Display,
{
    fmt: IndentWriter<W>,
    lalr1a: &'a LALR1A,
    grammar: &'a Grammar,
    labelling: F,
}

impl<'a, W, F, T> LALR1ADotWriter<'a, W, F, T>
where
    W: Write,
    F: Fn(Symbol) -> T,
    T: std::fmt::Display,
{
    #[must_use]
    pub fn new(fmt: W, lalr1a: &'a LALR1A, grammar: &'a Grammar, labelling: F) -> Self {
        Self {
            fmt: IndentWriter::new(fmt),
            lalr1a,
            grammar,
            labelling,
        }
    }

    /// # Errors
    pub fn build(mut self) -> Result<W, std::fmt::Error> {
        let states = self.lalr1a.states();

        writeln!(self.fmt, "digraph CC {{")?;
        self.fmt.indent();
        writeln!(self.fmt, "rankdir=LR;")?;

        writeln!(self.fmt)?;

        writeln!(self.fmt, "node[shape=point]; q;")?;
        writeln!(self.fmt, "node[shape=plain]; accept[label=<<b>ACCEPT</b>>];")?;
        for (id, state) in states.iter().enumerate() {
            self.format_state(id, state)?;
        }

        writeln!(self.fmt)?;

        writeln!(self.fmt, "q->s0;")?;
        writeln!(self.fmt, "s{}->accept[label=\"&#9633;\"];", *states[0].next.get(&Symbol::Variable(0)).unwrap())?;
        for (A, state) in states.iter().enumerate() {
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
                        let prod_a = self.grammar.productions().get(a.production);
                        let prod_b = self.grammar.productions().get(b.production);
                        let c1 = prod_a.0 == self.grammar.rules().len() - 1;
                        let c2 = prod_b.0 == self.grammar.rules().len() - 1;
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
                writeln!(self.fmt, "<tr><td align=\"left\">{}</td></tr>", self.format_item(id, item))?;   
            } else {
                writeln!(self.fmt, "<tr><td bgcolor=\"grey\" align=\"left\">{}</td></tr>", self.format_item(id, item))?;
            }
        }
        self.fmt.unindent();
        writeln!(self.fmt, "</table>>];")?;
        self.fmt.unindent();
        Ok(())
    }

    fn format_item(&self, state: usize, item: &LR0Item) -> String {
        let (lhs, rhs) = self.grammar.productions().get(item.production);
        
        let prefix = format!("{} &rarr; {}&bull;{}",
            (self.labelling)(Symbol::Variable(lhs)),
            rhs[..item.pos].iter().map(|&symbol| (self.labelling)(symbol).to_string()).collect::<Vec<_>>().join(" "),
            rhs[item.pos..].iter().map(|&symbol| (self.labelling)(symbol).to_string()).collect::<Vec<_>>().join(" "),
        );

        let state_reduction_pair = StateReductionPair { state, production: item.production };

        if let Some(lookahead) = self.lalr1a.lookahead.get(&state_reduction_pair) {
            let suffix =
                lookahead.iter().map(|&o| {
                    o.map_or("$".to_string(), |a| format!("{}", (self.labelling)(Symbol::Terminal(a))))
                }).collect::<Vec<_>>().join(" ");
            format!("{} [{}]", prefix, suffix)
        } else {
            prefix
        }
    }
}