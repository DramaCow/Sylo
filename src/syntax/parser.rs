use super::{
    LexerBuilder, Lexer, CST, CSTBuilder,
};
use crate::lang::{
    re::{Token, ScanError},
    cfg::{Grammar, Symbol},
    lr::{Event, Parse, ParseError, NaiveLRTable, ConstructionError, Conflict, Action},
};

pub struct ParserBuilder {
    pub lexer_def: LexerBuilder,
    pub var_names: Vec<String>,
    pub grammar: Grammar,
    pub token_precedence: Vec<Option<Precedence>>,
    pub production_precedence: Vec<Option<Precedence>>,
}

#[derive(Debug, Clone)]
pub struct Precedence {
    pub level: usize,
    pub associativity: Associativity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Associativity {
    Left,
    Right,
    Nonassoc,
}

pub struct Parser {
    pub lexer: Lexer,
    pub var_names: Vec<String>,
    pub parsing_table: NaiveLRTable,
}

impl ParserBuilder {
    #[must_use]
    pub fn new(lexer_def: LexerBuilder, var_names: Vec<String>, grammar: Grammar) -> Self {
        let word_count = grammar.word_count();
        let production_count = grammar.production_count();

        Self {
            lexer_def,
            var_names,
            grammar,
            token_precedence: vec![None; word_count],
            production_precedence: vec![None; production_count],
        }
    }

    pub fn set_token_precedence(&mut self, word: usize, precedence: Precedence) -> &mut Self {
        self.token_precedence[word] = Some(precedence);
        self
    }

    pub fn set_production_precedence(&mut self, production: usize, precedence: Precedence) -> &mut Self {
        self.production_precedence[production] = Some(precedence);
        self
    }

    /// # Errors
    pub fn build(mut self) -> Result<Parser, ConstructionError> {
        // In case where no production precedence has been specified, production precedence
        // is defaulted to the precedence of the rightmost token (that has some precedence).
        for (i, alt) in self.grammar.rules().flat_map(|rule| rule.alts()).enumerate() {
            if self.production_precedence[i].is_none() {
                self.production_precedence[i] = alt.iter().rev().find_map(|&symbol| {
                    if let Symbol::Terminal(a) = symbol {
                        self.token_precedence[a].clone()
                    } else {
                        None
                    }
                });
            }
        }

        Ok(Parser {
            lexer: self.lexer_def.compile(),
            var_names: self.var_names.to_vec(),
            parsing_table: NaiveLRTable::with_conflict_resolution(&self.grammar, |conflict| {
                match conflict {
                    Conflict::ShiftReduce { word, next_state, production } => {
                        // let tok  = if let Some(tok) = self.token_precedence[word].as_ref() { tok } else { return Err(conflict) };
                        // let prod = if let Some(prod) = self.production_precedence[alt].as_ref() { prod } else { return Err(conflict) };
                        let tok  = if let Some(tok) = self.token_precedence[word].as_ref() { tok } else { return Ok(Action::Shift(next_state)) };
                        let prod = if let Some(prod) = self.production_precedence[production].as_ref() { prod } else { return Ok(Action::Shift(next_state)) };

                        if prod.level > tok.level || (prod.level == tok.level && prod.associativity == Associativity::Left) {
                            Ok(Action::Reduce(production))
                        } else {
                            Ok(Action::Shift(next_state))
                        }
                    }
                    Conflict::ReduceReduce { .. } => {
                        Err(conflict)
                    }
                }
            })?,
        })
    }
}

impl<'a> Parser {
    /// # Errors
    // pub fn parse<I>(&'a self, input: &'a I) -> Parse<LR1Table, Scan<'a, LexTable, I>, Token<'a, I>, impl Fn(&Token<'a, I>) -> usize>
    pub fn parse<I>(&'a self, input: &'a I) -> impl Iterator<Item = Result<Event<Token<'a, I>>, ParseError<ScanError>>>
    where
        I: AsRef<[u8]> + ?Sized
    {
        Parse::new(&self.parsing_table, self.lexer.scan(input), |token: &Token<'a, I>| token.class)
    }

    /// # Errors
    pub fn cst(&'a self, text: &'a str) -> Result<CST, ParseError<ScanError>> {
        let mut builder = CSTBuilder::new();

        for res in self.parse(text) {
            match res? {
                Event::Shift(token) => builder.leaf(token.class, &text[token.span]),
                Event::Reduce { var, child_count, production: _ } => builder.branch(var, child_count),
            }
        }

        Ok(builder.build())
    }
}