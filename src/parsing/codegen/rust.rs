use std::fmt::Write;
use crate::utils::IndentWriter;
use crate::langcore::re::{LexTable, Command};
use crate::lexer::Lexer;
use super::rep;

/// # Errors
#[allow(clippy::too_many_lines)]
pub fn lexer<W: Write>(fmt: W, name: &str, lexer: &Lexer) -> Result<W, std::fmt::Error> {
    let mut fmt = IndentWriter::new(fmt);
    let ttypes: Vec<_> = lexer.vocab.iter().map(|s| s.to_ascii_uppercase()).collect();

    writeln!(fmt, "use std::ops::Range;")?;
    writeln!(fmt)?;
    writeln!(fmt, "pub fn scan<I: AsRef<[u8]> + ?Sized>(input: &I) -> Scan<'_> {{")?;
    writeln!(fmt, "    Scan {{")?;
    writeln!(fmt, "        input: input.as_ref(),")?;
    writeln!(fmt, "        index: 0,")?;
    writeln!(fmt, "    }}")?;
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "pub struct Scan<'a> {{")?;
    writeln!(fmt, "    input: &'a [u8],")?;
    writeln!(fmt, "    index: usize,")?;
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "#[derive(Debug)]")?;
    writeln!(fmt, "pub struct ScanError {{")?;
    writeln!(fmt, "    pub pos: usize,")?;
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "#[derive(Debug)]")?;
    writeln!(fmt, "pub enum TokenType {{")?;
    for (_, ttype) in ttypes.iter().enumerate().filter(|(i, _)| if let Command::Emit = lexer.table.commands[*i] { true } else { false }) {
        writeln!(fmt, "    {ttype},", ttype=ttype)?;
    }
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "#[derive(Debug)]")?;
    writeln!(fmt, "pub struct Token {{")?;
    writeln!(fmt, "    pub ttype: TokenType,")?;
    writeln!(fmt, "    pub span: Range<usize>,")?;
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "impl<'a> Iterator for Scan<'a> {{")?;
    writeln!(fmt, "    type Item = Result<Token, ScanError>;")?;
    writeln!(fmt)?;
    writeln!(fmt, "    fn next(&mut self) -> Option<Self::Item> {{")?;
    writeln!(fmt, "        self.begin()")?;
    writeln!(fmt, "    }}")?;
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "enum LastAcceptTType {{")?;
    writeln!(fmt, "    Tok(TokenType),")?;
    writeln!(fmt, "    Error,")?;
    writeln!(fmt, "    Skip,")?;
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "struct Context {{")?;
    writeln!(fmt, "    start_index: usize,")?;
    writeln!(fmt, "    last_accept_ttype: LastAcceptTType,")?;
    writeln!(fmt, "    last_accept_index: usize,")?;
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "impl Scan<'_> {{")?;
    writeln!(fmt, "    fn begin(&mut self) -> Option<<Self as Iterator>::Item> {{")?;
    writeln!(fmt, "        if self.index >= self.input.len() {{")?;
    writeln!(fmt, "            return None;")?;
    writeln!(fmt, "        }}")?;
    writeln!(fmt)?;
    writeln!(fmt, "        let ctx = Context {{")?;
    writeln!(fmt, "            start_index: self.index,")?;
    writeln!(fmt, "            last_accept_ttype: LastAcceptTType::Error,")?;
    writeln!(fmt, "            last_accept_index: 0,")?;
    writeln!(fmt, "        }};")?;
    writeln!(fmt)?;
    writeln!(fmt, "        self.s0(ctx)")?;
    writeln!(fmt, "    }}")?;
    writeln!(fmt)?;
    writeln!(fmt, "// *** LEXER TABLE START ***")?;
    writeln!(fmt)?;
    fmt.indent();
    for (i, state) in rep::Lexer::new(name, lexer).states.iter().enumerate() {
        writeln!(fmt, "fn s{}(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {{", i)?;
        fmt.indent();

        if state.transitions.is_empty() {
            // `i` is a labelled state
            if let Some(class) = state.class {
                if let Command::Skip = lexer.table.command(class) {
                    writeln!(fmt, "self.begin()")?;
                } else {
                    writeln!(fmt, "Some(Ok(Token {{ ttype: TokenType::{ttype}, span: ctx.start_index..self.index }}))", ttype=ttypes[class])?;
                }
            } else {
                writeln!(fmt, "self.sink(ctx)")?;
            }
        } else {
            writeln!(fmt, "if self.index >= self.input.len() {{ return self.sink(ctx); }}")?;
            writeln!(fmt, "let ch = self.input[self.index];")?;
            // writeln!(fmt, "NEXT_CHAR(ch)")?;

            // `i` is a labelled state
            if let Some(class) = state.class {
                let ttype = if let Command::Skip = lexer.table.command(class) {
                    "LastAcceptTType::Skip".to_string()
                } else {
                    format!("LastAcceptTType::Tok(TokenType::{ttype})", ttype=ttypes[class])
                };

                // If `i` can only transition to labelled states, then either
                // its corresponding token will be returned immediately or the
                // token of any destination state will take priority (due to
                // maximal munch). As such, there would be no need for the
                // last accept token type or index to be stored. 
                if state.can_transition_to_unlabelled_state {
                    writeln!(fmt, "ctx.last_accept_ttype = {ttype};", ttype=ttype)?;
                    writeln!(fmt, "ctx.last_accept_index = self.index;")?;
                }
            }

            // Transitions to non-sink states. Semantically speaking, if any no
            // transition is taken, then we transition to the sink state.
            for rep::Transition { intervals, dst } in &state.transitions {
                write!(fmt, "if ")?;

                for (i, &(start, end)) in intervals.iter().enumerate() {
                    if i > 0 {
                        write!(fmt, "   ")?;
                    }

                    #[allow(clippy::comparison_chain)]
                    if start + 1 < end {
                        write!(fmt, "(0x{:02x?} <= ch && ch <= 0x{:02x?})", start, end)?;
                    } else if start + 1 == end {
                        writeln!(fmt, "ch == 0x{:02x?} ||", start)?;
                        write!(fmt, "   ch == 0x{:02x?}", end)?;
                    } else {
                        write!(fmt, "ch == 0x{:02x?}", start)?;
                    }

                    if i < intervals.len() - 1 {
                        writeln!(fmt, " ||")?;
                    }
                }

                writeln!(fmt, " {{ self.index += 1; return self.s{dst}(ctx); }}", dst=dst)?;
                // writeln!(fmt, ") GOTO({});", dst)?;
            }
            
            // `i` is a labelled state
            if let Some(class) = state.class {
                if let Command::Skip = lexer.table.command(class) {
                    writeln!(fmt, "self.begin()")?;
                } else {
                    writeln!(fmt, "Some(Ok((Token {{ ttype: TokenType::{ttype}, span: ctx.start_index..self.index }}))", ttype=ttypes[class])?;
                }
            } else {
                writeln!(fmt, "self.sink(ctx)")?;
            }
        }

        fmt.unindent();
        writeln!(fmt, "}}\n")?;
    }
    fmt.unindent();
    writeln!(fmt, "// ***  LEXER TABLE END  ***")?;
    writeln!(fmt)?;
    writeln!(fmt, "    fn sink(&mut self, ctx: Context) -> Option<<Self as Iterator>::Item> {{")?;
    writeln!(fmt, "        match ctx.last_accept_ttype {{")?;
    writeln!(fmt, "            LastAcceptTType::Tok(ttype) => {{")?;
    writeln!(fmt, "                self.index = ctx.last_accept_index;")?;
    writeln!(fmt, "                Some(Ok(Token {{ ttype, span: ctx.start_index..ctx.last_accept_index }}))")?;
    writeln!(fmt, "            }}")?;
    writeln!(fmt, "            LastAcceptTType::Error => {{")?;
    writeln!(fmt, "                let pos = self.index;")?;
    writeln!(fmt, "                self.index = usize::MAX; // forces next iteration to return None")?;
    writeln!(fmt, "                Some(Err(ScanError {{ pos }}))")?;
    writeln!(fmt, "            }}")?;
    writeln!(fmt, "            LastAcceptTType::Skip => {{")?;
    writeln!(fmt, "                self.index = ctx.last_accept_index;")?;
    writeln!(fmt, "                self.begin()")?;
    writeln!(fmt, "            }}")?;
    writeln!(fmt, "        }}")?;
    writeln!(fmt, "    }}")?;
    writeln!(fmt, "}}")?;
    
    Ok(fmt.build())
}