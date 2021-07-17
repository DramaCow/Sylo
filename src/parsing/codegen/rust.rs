use std::fmt::Write;
use crate::utils::IndentWriter;
use crate::langcore::re::Command;
use crate::langcore::lr1_table::Action;
use crate::lexer::Lexer;
use crate::parser::Parser;
use super::{rep, par};

/// # Errors
#[allow(clippy::too_many_lines)]
pub fn lexer<W: Write>(fmt: W, name: &str, lexer: &Lexer) -> Result<W, std::fmt::Error> {
    let lexer = rep::Lexer::new(name, lexer);

    let mut fmt = IndentWriter::new(fmt);

    writeln!(fmt, "/* automatically generated by sylo */")?;
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
    writeln!(fmt, "#[derive(Debug, Clone, Copy)]")?;
    writeln!(fmt, "pub enum TokenType {{")?;
    for ttype in lexer.ttypes.iter().zip(&lexer.commands).filter_map(|(ttype, command)| if let Command::Emit = command { Some(ttype) } else { None }) {
        writeln!(fmt, "    {ttype},", ttype=ttype)?;
    }
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "#[derive(Debug, Clone, Copy)]")?;
    writeln!(fmt, "pub struct Token {{")?;
    writeln!(fmt, "    pub ttype: TokenType,")?;
    writeln!(fmt, "    pub span: (usize, usize),")?;
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
    for (i, state) in lexer.states.iter().enumerate() {
        writeln!(fmt, "fn s{}(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {{", i)?;
        fmt.indent();

        if state.transitions.is_empty() {
            // `i` is a labelled state
            if let Some(class) = state.class {
                if let Command::Skip = lexer.commands[class] {
                    writeln!(fmt, "self.begin()")?;
                } else {
                    writeln!(fmt, "Some(Ok(Token {{ ttype: TokenType::{ttype}, span: (ctx.start_index, self.index) }}))", ttype=lexer.ttypes[class])?;
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
                let ttype = if let Command::Skip = lexer.commands[class] {
                    "LastAcceptTType::Skip".to_string()
                } else {
                    format!("LastAcceptTType::Tok(TokenType::{ttype})", ttype=lexer.ttypes[class])
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
                if let Command::Skip = lexer.commands[class] {
                    writeln!(fmt, "self.begin()")?;
                } else {
                    writeln!(fmt, "Some(Ok((Token {{ ttype: TokenType::{ttype}, span: (ctx.start_index, self.index) }}))", ttype=lexer.ttypes[class])?;
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
    writeln!(fmt, "                Some(Ok(Token {{ ttype, span: (ctx.start_index, ctx.last_accept_index) }}))")?;
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

/// # Errors
#[allow(clippy::too_many_lines)]
pub fn parser<W: Write>(fmt: W, name: &str, parser: &Parser) -> Result<W, std::fmt::Error> {
    let mut fmt = IndentWriter::new(lexer(fmt, name, &parser.lexer)?);
    writeln!(fmt)?;

    let parser = par::LR1Parser::new(name, parser);
    
    writeln!(fmt, "pub fn parse<I: AsRef<[u8]> + ?Sized>(input: &I) -> Result<Variable, ParseError> {{")?;
    writeln!(fmt, "    Parse {{ input: scan(input), next_token: None }}.begin()")?;
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "#[derive(Debug, Clone, Copy)]")?;
    writeln!(fmt, "pub enum Variable {{")?;
    for varname in &parser.varnames {
        writeln!(fmt, "    {varname}(i32),", varname=varname)?; // TODO: product type is configurable per variable
    }
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "pub struct Parse<'a> {{")?;
    writeln!(fmt, "    input: Scan<'a>,")?;
    writeln!(fmt, "    next_token: Option<Token>,")?;
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "#[derive(Debug)]")?;
    writeln!(fmt, "pub enum ParseError {{")?;
    writeln!(fmt, "    InputError(ScanError),")?;
    writeln!(fmt, "    InvalidAction {{ state: usize, ttype: Option<TokenType> }},")?;
    writeln!(fmt, "    InvalidGoto {{ state: usize }},")?;
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "impl Parse<'_> {{")?;
    writeln!(fmt, "    fn begin(mut self) -> Result<Variable, ParseError> {{")?;
    writeln!(fmt, "        self.update()?;")?;
    writeln!(fmt, "        Ok(self.s0()?.0)")?;
    writeln!(fmt, "    }}")?;
    writeln!(fmt)?;
    writeln!(fmt, "    fn update(&mut self) -> Result<(), ParseError> {{")?;
    writeln!(fmt, "        self.next_token = self.input.next().transpose().map_err(|err| {{ ParseError::InputError(err) }})?;")?;
    writeln!(fmt, "        Ok(())")?;
    writeln!(fmt, "    }}")?;
    writeln!(fmt)?;
    fmt.indent();
    let mut uses_decrement = false;
    for (i, state) in parser.states.iter().enumerate() {
        writeln!(fmt, "fn s{}(&mut self) -> Result<(Variable, usize), ParseError> {{", i)?;
        // writeln!(fmt, "    print!(\"{state} \");", state=i)?;
        fmt.indent();
        if !state.ttrans.is_empty() {
            if state.has_shift_transitions {
                if state.nttrans.is_empty() {
                    uses_decrement = true;
                } else {
                    writeln!(fmt, "fn on_return(parse: &mut Parse, (var, goto): (Variable, usize)) -> Result<(Variable, usize), ParseError> {{")?;
                    writeln!(fmt, "    if goto == 0 {{")?;
                    writeln!(fmt, "        let tuple = match var {{")?;
                    for nttran in &state.nttrans {
                        writeln!(fmt, "            Variable::{varname}(_) => parse.s{dst}(),", varname=parser.varnames[nttran.var], dst=nttran.dst)?;
                    }
                    if state.nttrans.len() < parser.varnames.len() {
                        writeln!(fmt, "            _ => return Err(ParseError::InvalidGoto {{ state: {state} }}),", state=i)?;
                    }
                    writeln!(fmt, "        }}?;")?;
                    writeln!(fmt, "        on_return(parse, tuple)")?;
                    writeln!(fmt, "    }} else {{")?;
                    writeln!(fmt, "        Ok((var, goto - 1))")?;
                    writeln!(fmt, "    }}")?;
                    writeln!(fmt, "}}")?;
                }
                write!(fmt, "let tuple = ")?;   
            }

            writeln!(fmt, "match self.next_token.map(|token| token.ttype) {{")?;
            for ttran in &state.ttrans {
                if let Some(word) = ttran.word {
                    write!(fmt, "    Some(TokenType::{ttype}) => ", ttype=parser.ttypes[word])?;
                } else {
                    write!(fmt, "    None => ")?;
                }
                match ttran.action {
                    Action::Invalid => panic!(),
                    Action::Accept => { writeln!(fmt, "return Ok((Variable::{varname}(0), 1)),", varname=parser.varnames[0])?; } // TODO:
                    Action::Shift(dst) => { writeln!(fmt, "{{ self.update()?; self.s{dst}() }}", dst=dst)?; }
                    Action::Reduce(p) => {
                        let r = parser.reductions[p];
                        writeln!(fmt, "return Ok((Variable::{varname}(0), {count})),", varname=parser.varnames[r.var], count=r.count-1)?;
                    }
                };
            }
            if state.ttrans.len() < parser.ttype_count {
                writeln!(fmt, "    ttype => return Err(ParseError::InvalidAction {{ state: {state}, ttype }}),", state=i)?;
            }
            write!(fmt, "}}")?;

            if state.has_shift_transitions {
                writeln!(fmt, "?;")?;
                if state.nttrans.is_empty() {
                    writeln!(fmt, "self.decrement(tuple)")?;
                } else {
                    writeln!(fmt, "on_return(self, tuple)")?;
                }
            } else {
                writeln!(fmt)?;
            }
        }

        fmt.unindent();
        writeln!(fmt, "}}")?;

        if i < parser.states.len() - 1 {
            writeln!(fmt);
        }
    }
    if uses_decrement {
        writeln!(fmt, "\nfn decrement(&mut self, (var, goto): (Variable, usize)) -> Result<(Variable, usize), ParseError> {{")?;
        writeln!(fmt, "    assert!(goto > 0);")?;
        writeln!(fmt, "    Ok((var, goto - 1))")?;
        writeln!(fmt, "}}")?;
    }
    fmt.unindent();
    writeln!(fmt, "}}")?;
    
    Ok(fmt.build())
}