#![allow(clippy::match_on_vec_items)]
#![allow(non_snake_case)]

use std::fmt::Write;
use crate::utils::IndentWriter;
use crate::langcore::cfg::Symbol;
use crate::langcore::re::Command;
use crate::langcore::lr1_table::Action;
use super::ir;

pub struct RustWriter<W: Write> {
    fmt: IndentWriter<W>,
}

impl<W: Write> RustWriter<W> {
    #[must_use]
    pub fn new(fmt: W) -> Self {
        Self { fmt: IndentWriter::new(fmt) }
    }

    #[must_use]
    pub fn build(self) -> W {
        self.fmt.build()
    }

    /// # Errors
    pub fn lexer(mut self, lexer: &ir::Lexer) -> Result<Self, std::fmt::Error> {
        writeln!(self.fmt, "/* automatically generated by sylo */")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "pub fn scan<I: AsRef<[u8]> + ?Sized>(input: &I) -> Scan<'_> {{")?;
        writeln!(self.fmt, "    Scan {{")?;
        writeln!(self.fmt, "        input: input.as_ref(),")?;
        writeln!(self.fmt, "        index: 0,")?;
        writeln!(self.fmt, "    }}")?;
        writeln!(self.fmt, "}}")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "pub struct Scan<'a> {{")?;
        writeln!(self.fmt, "    input: &'a [u8],")?;
        writeln!(self.fmt, "    index: usize,")?;
        writeln!(self.fmt, "}}")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "#[derive(Debug)]")?;
        writeln!(self.fmt, "pub struct ScanError {{")?;
        writeln!(self.fmt, "    pub pos: usize,")?;
        writeln!(self.fmt, "}}")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "#[derive(Debug, Clone, Copy)]")?;
        writeln!(self.fmt, "pub enum TokenType {{")?;
        for ttype in lexer.ttypes.iter().zip(&lexer.commands).filter_map(|(ttype, command)| if let Command::Emit = command { Some(ttype) } else { None }) {
            writeln!(self.fmt, "    {ttype},", ttype=ttype)?;
        }
        writeln!(self.fmt, "}}")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "#[derive(Debug, Clone, Copy)]")?;
        writeln!(self.fmt, "pub struct Token {{")?;
        writeln!(self.fmt, "    pub ttype: TokenType,")?;
        writeln!(self.fmt, "    pub span: (usize, usize),")?;
        writeln!(self.fmt, "}}")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "impl<'a> Iterator for Scan<'a> {{")?;
        writeln!(self.fmt, "    type Item = Result<Token, ScanError>;")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "    fn next(&mut self) -> Option<Self::Item> {{")?;
        writeln!(self.fmt, "        self.begin()")?;
        writeln!(self.fmt, "    }}")?;
        writeln!(self.fmt, "}}")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "enum LastAcceptTType {{")?;
        writeln!(self.fmt, "    Tok(TokenType),")?;
        writeln!(self.fmt, "    Error,")?;
        writeln!(self.fmt, "    Skip,")?;
        writeln!(self.fmt, "}}")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "struct Context {{")?;
        writeln!(self.fmt, "    start_index: usize,")?;
        writeln!(self.fmt, "    last_accept_ttype: LastAcceptTType,")?;
        writeln!(self.fmt, "    last_accept_index: usize,")?;
        writeln!(self.fmt, "}}")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "impl Scan<'_> {{")?;
        writeln!(self.fmt, "    fn begin(&mut self) -> Option<<Self as Iterator>::Item> {{")?;
        writeln!(self.fmt, "        if self.index >= self.input.len() {{")?;
        writeln!(self.fmt, "            return None;")?;
        writeln!(self.fmt, "        }}")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "        let ctx = Context {{")?;
        writeln!(self.fmt, "            start_index: self.index,")?;
        writeln!(self.fmt, "            last_accept_ttype: LastAcceptTType::Error,")?;
        writeln!(self.fmt, "            last_accept_index: 0,")?;
        writeln!(self.fmt, "        }};")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "        self.s0(ctx)")?;
        writeln!(self.fmt, "    }}")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "// *** LEXER TABLE START ***")?;
        writeln!(self.fmt)?;
        {
            self.fmt.indent();
            for state in &lexer.states {
                self.lexer_state(lexer, state)?;
            }
            self.fmt.unindent();
        }
        writeln!(self.fmt, "// ***  LEXER TABLE END  ***")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "    fn sink(&mut self, ctx: &Context) -> Option<<Self as Iterator>::Item> {{")?;
        writeln!(self.fmt, "        match ctx.last_accept_ttype {{")?;
        writeln!(self.fmt, "            LastAcceptTType::Tok(ttype) => {{")?;
        writeln!(self.fmt, "                self.index = ctx.last_accept_index;")?;
        writeln!(self.fmt, "                Some(Ok(Token {{ ttype, span: (ctx.start_index, ctx.last_accept_index) }}))")?;
        writeln!(self.fmt, "            }}")?;
        writeln!(self.fmt, "            LastAcceptTType::Error => {{")?;
        writeln!(self.fmt, "                let pos = self.index;")?;
        writeln!(self.fmt, "                self.index = usize::MAX; // forces next iteration to return None")?;
        writeln!(self.fmt, "                Some(Err(ScanError {{ pos }}))")?;
        writeln!(self.fmt, "            }}")?;
        writeln!(self.fmt, "            LastAcceptTType::Skip => {{")?;
        writeln!(self.fmt, "                self.index = ctx.last_accept_index;")?;
        writeln!(self.fmt, "                self.begin()")?;
        writeln!(self.fmt, "            }}")?;
        writeln!(self.fmt, "        }}")?;
        writeln!(self.fmt, "    }}")?;
        writeln!(self.fmt, "}}")?;

        Ok(self)
    }

    /// # Errors
    /// # Panics
    pub fn parser(mut self, parser: &ir::Parser) -> Result<Self, std::fmt::Error> {
        writeln!(self.fmt, "#![allow(non_camel_case_types)]")?;
        writeln!(self.fmt, "#![allow(unused_comparisons)]")?;
        writeln!(self.fmt, "#![allow(dead_code)]")?;
        writeln!(self.fmt, "#![allow(non_snake_case)]")?;
        writeln!(self.fmt, "#![allow(unused_mut)]")?;
        writeln!(self.fmt, "#![allow(clippy::match_same_arms)]")?;
        writeln!(self.fmt, "#![allow(clippy::needless_return)]")?;
        writeln!(self.fmt, "#![allow(clippy::unnecessary_wraps)]")?;
        writeln!(self.fmt, "#![allow(clippy::unit_arg)]")?;
        writeln!(self.fmt)?;
        for varname in &parser.varnames {
            writeln!(self.fmt, "type {varname}Product = ();", varname=varname)?; // TODO: product type is configurable per variable
        }
        writeln!(self.fmt)?;
        for (i, (lhs, rhs)) in parser.grammar.productions().into_iter().enumerate() {
            if lhs < parser.grammar.rules().len() - 1 { // ignore the augmented grammar rule
                // write comment
                write!(self.fmt, "// {varname} ->", varname=parser.varnames[lhs])?;
                for &symbol in rhs {
                    match symbol {
                        Symbol::Terminal(a) => write!(self.fmt, " {}", parser.lexer.ttypes[a])?,
                        Symbol::Variable(A) => write!(self.fmt, " {}", parser.varnames[A])?,
                    }
                }
                writeln!(self.fmt)?;

                // write function signature
                write!(self.fmt, "fn p{i}(", i=i)?;
                for (i, &symbol) in rhs.iter().enumerate() {
                    match symbol {
                        Symbol::Terminal(_) => write!(self.fmt, "_: &[u8]")?,
                        Symbol::Variable(A) => write!(self.fmt, "_: {varname}Product", varname=parser.varnames[A])?,
                    }
                    if i < rhs.len()-1 {
                        write!(self.fmt, ", ")?;
                    }
                }
                writeln!(self.fmt, ") -> {varname}Product {{", varname=parser.varnames[lhs])?;
                writeln!(self.fmt, "    todo!()")?;
                writeln!(self.fmt, "}}\n")?;
            }
        }
        self = self.lexer(&parser.lexer)?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "pub fn parse<I: AsRef<[u8]> + ?Sized>(input: &I) -> Result<{varname}Product, ParseError> {{", varname=parser.varnames[0])?;
        writeln!(self.fmt, "    let res = Parse {{ input: scan(input), next_token: None }}.begin();")?;
        writeln!(self.fmt, "    match res? {{")?;
        writeln!(self.fmt, "        Variable::{varname}(product) => Ok(product),", varname=parser.varnames[0])?;
        writeln!(self.fmt, "        _ => unreachable!(),")?;
        writeln!(self.fmt, "    }}")?;
        writeln!(self.fmt, "}}")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "#[derive(Debug, Clone, Copy)]")?;
        writeln!(self.fmt, "pub enum Variable {{")?;
        for varname in &parser.varnames {
            writeln!(self.fmt, "    {varname}({varname}Product),", varname=varname)?;
        }
        writeln!(self.fmt, "}}")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "pub struct Parse<'a> {{")?;
        writeln!(self.fmt, "    input: Scan<'a>,")?;
        writeln!(self.fmt, "    next_token: Option<Token>,")?;
        writeln!(self.fmt, "}}")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "#[derive(Debug)]")?;
        writeln!(self.fmt, "pub enum ParseError {{")?;
        writeln!(self.fmt, "    InputError(ScanError),")?;
        writeln!(self.fmt, "    InvalidAction {{ state: usize, ttype: Option<TokenType> }},")?;
        writeln!(self.fmt, "    InvalidGoto {{ state: usize }},")?;
        writeln!(self.fmt, "}}")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "impl Parse<'_> {{")?;
        writeln!(self.fmt, "    fn begin(mut self) -> Result<Variable, ParseError> {{")?;
        writeln!(self.fmt, "        self.update()?;")?;
        writeln!(self.fmt, "        Ok(self.s0()?.0)")?;
        writeln!(self.fmt, "    }}")?;
        writeln!(self.fmt)?;
        writeln!(self.fmt, "    fn update(&mut self) -> Result<(), ParseError> {{")?;
        writeln!(self.fmt, "        self.next_token = self.input.next().transpose().map_err(|err| {{ ParseError::InputError(err) }})?;")?;
        writeln!(self.fmt, "        Ok(())")?;
        writeln!(self.fmt, "    }}")?;
        writeln!(self.fmt)?;        
        {
            self.fmt.indent();
            for state in &parser.states {
                self.parser_state(parser, state)?;
            }
            writeln!(self.fmt, "\nfn lexeme(&self, token: &Token) -> &[u8] {{")?;
            writeln!(self.fmt, "    &self.input.input[token.span.0..token.span.1]")?;
            writeln!(self.fmt, "}}")?;
            self.fmt.unindent();
        }
        writeln!(self.fmt, "}}")?;
        writeln!(self.fmt, "\nfn decrement((var, goto): (Variable, usize)) -> Result<(Variable, usize), ParseError> {{")?;
        writeln!(self.fmt, "    assert!(goto > 0);")?;
        writeln!(self.fmt, "    Ok((var, goto - 1))")?;
        writeln!(self.fmt, "}}")?;

        Ok(self)
    }
}

// =================
// === INTERNALS ===
// =================

impl<W: Write> RustWriter<W> {
    /// # Errors
    /// # Panics
    #[allow(clippy::too_many_lines)]
    fn lexer_state(&mut self, lexer: &ir::Lexer, state: &ir::LexerState) -> Result<(), std::fmt::Error> {
        writeln!(self.fmt, "fn s{}(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {{", state.id)?;
        {
            self.fmt.indent();

            if !state.transitions.is_empty() {
                writeln!(self.fmt, "if self.index < self.input.len() {{")?;
                {
                    self.fmt.indent();

                    writeln!(self.fmt, "let ch = self.input[self.index];")?;

                    // `i` is a labelled state
                    if let Some(class) = state.class {
                        // If `i` can only transition to labelled states, then either
                        // its corresponding token will be returned immediately or the
                        // token of any destination state will take priority (due to
                        // maximal munch). As such, there would be no need for the
                        // last accept token type or index to be stored. 
                        if state.can_transition_to_unlabelled_state {
                            if let Command::Skip = lexer.commands[class] {
                                writeln!(self.fmt, "ctx.last_accept_ttype = LastAcceptTType::Skip;")?;
                            } else {
                                writeln!(self.fmt, "ctx.last_accept_ttype = LastAcceptTType::Tok(TokenType::{ttype});", ttype=lexer.ttypes[class])?;
                            };
                            writeln!(self.fmt, "ctx.last_accept_index = self.index;")?;
                        }
                    }

                    // Transitions to non-sink states. Semantically speaking, if any no
                    // transition is taken, then we transition to the sink state.
                    for ir::Transition { intervals, dst } in &state.transitions {
                        write!(self.fmt, "if ")?;

                        for (i, &(start, end)) in intervals.iter().enumerate() {
                            if i > 0 {
                                write!(self.fmt, "   ")?;
                            }

                            #[allow(clippy::comparison_chain)]
                            if start + 1 < end {
                                write!(self.fmt, "(0x{:02x?}..=0x{:02x?}).contains(&ch)", start, end)?;
                            } else if start + 1 == end {
                                writeln!(self.fmt, "ch == 0x{:02x?} ||", start)?;
                                write!(self.fmt, "   ch == 0x{:02x?}", end)?;
                            } else {
                                write!(self.fmt, "ch == 0x{:02x?}", start)?;
                            }

                            if i < intervals.len() - 1 {
                                writeln!(self.fmt, " ||")?;
                            }
                        }

                        writeln!(self.fmt, " {{ self.index += 1; return self.s{dst}(ctx); }}", dst=dst)?;
                    }

                    self.fmt.unindent();
                }
                writeln!(self.fmt, "}}")?;        
            }

            // `i` is a labelled state
            if let Some(class) = state.class {
                if let Command::Skip = lexer.commands[class] {
                    writeln!(self.fmt, "self.begin()")?;
                } else {
                    writeln!(self.fmt, "Some(Ok(Token {{ ttype: TokenType::{ttype}, span: (ctx.start_index, self.index) }}))", ttype=lexer.ttypes[class])?;
                }
            } else {
                writeln!(self.fmt, "self.sink(&ctx)")?;
            }

            self.fmt.unindent();
        }
        writeln!(self.fmt, "}}\n")?;

        Ok(())
    }

    /// # Errors
    /// # Panics
    #[allow(clippy::too_many_lines)]
    fn parser_state(&mut self, parser: &ir::Parser, state: &ir::ParserState) -> Result<(), std::fmt::Error> {
        let arg = match state.input_symbols.len() {
            x if x > 1 => {
                let arg_types: Vec<_> = state.input_symbols.iter().map(|&symbol| {
                    match symbol {
                        Symbol::Terminal(_) => "Token".to_string(),
                        Symbol::Variable(A) => format!("{varname}Product", varname=parser.varnames[A]),
                    }
                }).collect();
                Some(("args", format!("({})", arg_types.join(", "))))
            }
            x if x == 1 => {
                Some(("arg", match state.input_symbols[0] {
                    Symbol::Terminal(_) => "Token".to_string(),
                    Symbol::Variable(A) => format!("{varname}Product", varname=parser.varnames[A]),
                }))
            }
            _ => {
                None
            }
        };

        // state function signature
        match arg {
            Some((name, ref r#type)) => writeln!(self.fmt, "fn s{}(&mut self, {argname}: {argtype}) -> Result<(Variable, usize), ParseError> {{", state.id, argname=name, argtype=r#type)?,
            None => writeln!(self.fmt, "fn s{}(&mut self) -> Result<(Variable, usize), ParseError> {{", state.id)?,
        }
        self.fmt.indent();

        if !state.ttrans.is_empty() {
            if state.has_shift_transitions {
                if !state.nttrans.is_empty() {
                    // on_return function definition
                    match arg {
                        Some((name, ref r#type)) => writeln!(self.fmt, "fn on_return(parse: &mut Parse, (var, goto): (Variable, usize), {argname}: {argtype}) -> Result<(Variable, usize), ParseError> {{", argname=name, argtype=r#type)?,
                        None => writeln!(self.fmt, "fn on_return(parse: &mut Parse, (var, goto): (Variable, usize)) -> Result<(Variable, usize), ParseError> {{")?,
                    }

                    {
                        self.fmt.indent();
                        writeln!(self.fmt, "if goto == 0 {{")?;
                        {
                            self.fmt.indent();
                            writeln!(self.fmt, "let tuple = match var {{")?;
                            for nttran in &state.nttrans {
                                let dst = nttran.dst;
                                write!(self.fmt, "    Variable::{varname}(v) => parse.s{dst}(", varname=parser.varnames[nttran.var], dst=dst)?;
                                self.arglist(state, &parser.states[dst], "v")?;
                                writeln!(self.fmt, "),")?;
                            }
                            if state.nttrans.len() < parser.varnames.len() {
                                writeln!(self.fmt, "    _ => return Err(ParseError::InvalidGoto {{ state: {state} }}),", state=state.id)?;
                            }
                            writeln!(self.fmt, "}}?;")?;
                            match arg {
                                Some((name, _)) => writeln!(self.fmt, "on_return(parse, tuple, {argname})", argname=name)?,
                                None => writeln!(self.fmt, "on_return(parse, tuple)")?,
                            }
                            self.fmt.unindent();
                        }
                        writeln!(self.fmt, "}} else {{")?;
                        writeln!(self.fmt, "    Ok((var, goto - 1))")?;
                        writeln!(self.fmt, "}}")?;
                        self.fmt.unindent();
                    }
                    
                    writeln!(self.fmt, "}}")?;
                }

                write!(self.fmt, "let tuple = ")?;   
            }

            writeln!(self.fmt, "match self.next_token {{")?;
            {
                self.fmt.indent();
                for ttran in &state.ttrans {
                    match ttran.action {
                        Action::Invalid => panic!(),
                        Action::Accept => {
                            match ttran.word {
                                Some(word) => write!(self.fmt, "Some(Token {{ ttype: TokenType::{ttype}, .. }}) => ", ttype=parser.lexer.ttypes[word])?,
                                None => write!(self.fmt, "None => ")?,
                            }
                            writeln!(self.fmt, "return Ok((Variable::{varname}(arg), 1)),", varname=parser.varnames[0])?;
                        }
                        Action::Shift(dst) => {
                            match ttran.word {
                                Some(word) => write!(self.fmt, "Some(t @ Token {{ ttype: TokenType::{ttype}, .. }}) => ", ttype=parser.lexer.ttypes[word])?,
                                None => write!(self.fmt, "None => ")?,
                            }
                            write!(self.fmt, "{{ self.update()?; self.s{dst}(", dst=dst)?;
                            self.arglist(state, &parser.states[dst], "t")?;
                            writeln!(self.fmt, ") }}")?;
                        }
                        Action::Reduce(p) => {
                            match ttran.word {
                                Some(word) => write!(self.fmt, "Some(Token {{ ttype: TokenType::{ttype}, .. }}) => ", ttype=parser.lexer.ttypes[word])?,
                                None => write!(self.fmt, "None => ")?,
                            }
                            let (lhs, rhs) = parser.grammar.productions().get(p);
                            write!(self.fmt, "return Ok((Variable::{varname}(p{i}(", varname=parser.varnames[lhs], i=p)?;
                            let input_symbols = &state.input_symbols;
                            let alt = parser.grammar.productions().get(p).1;
                            let offset = input_symbols.len() - alt.len();
                            for i in offset..input_symbols.len() {
                                let arg = if input_symbols.len() == 1 { "arg".to_string() } else { format!("args.{i}", i=i) };

                                match input_symbols[i] {
                                    Symbol::Terminal(_) => write!(self.fmt, "self.lexeme(&{arg})", arg=arg)?,
                                    Symbol::Variable(_) => write!(self.fmt, "{arg}", arg=arg)?,
                                }

                                if i < input_symbols.len() - 1 {
                                    write!(self.fmt, ", ")?;
                                }
                            }
                            writeln!(self.fmt, ")), {count})),", count=rhs.len()-1)?;
                        }
                    };
                }
                if state.ttrans.len() < parser.lexer.ttype_count {
                    writeln!(self.fmt, "t => return Err(ParseError::InvalidAction {{ state: {state}, ttype: t.map(|t| t.ttype) }}),", state=state.id)?;
                }
                self.fmt.unindent();
            }
            write!(self.fmt, "}}")?;

            if state.has_shift_transitions {
                writeln!(self.fmt, "?;")?;
                if state.nttrans.is_empty() {
                    writeln!(self.fmt, "decrement(tuple)")?;
                } else {
                    match arg {
                        Some((name, _)) => writeln!(self.fmt, "on_return(self, tuple, {argname})", argname=name)?,
                        None => writeln!(self.fmt, "on_return(self, tuple)")?,
                    }
                }
            } else {
                writeln!(self.fmt)?;
            }
        }

        self.fmt.unindent();
        writeln!(self.fmt, "}}")?;

        if state.id < parser.states.len() - 1 {
            writeln!(self.fmt)?;
        }

        Ok(())
    }

    fn arglist(&mut self, src: &ir::ParserState, dst: &ir::ParserState, arg: &str) -> Result<(), std::fmt::Error> {    
        let next_symbols = &dst.input_symbols;
        
        if next_symbols.len() > 1 {
            write!(self.fmt, "(")?;
        }
    
        let input_symbols = &src.input_symbols;
        let offset = input_symbols.len() - (next_symbols.len() - 1);
        
        for i in offset..input_symbols.len() {
            if src.input_symbols.len() == 1 {
                write!(self.fmt, "arg, ")?;
            } else {
                write!(self.fmt, "args.{i}, ", i=i)?;
            }
        }
        
        write!(self.fmt, "{}", arg)?;
        
        if next_symbols.len() > 1 {
            write!(self.fmt, ")")?;
        }
        
        Ok(())
    }
}
