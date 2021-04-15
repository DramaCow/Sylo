use super::{
    lex::{Token, Scan, ScanError, ArrayScanningTable},
    LexerDef,
    Lexer,
    cfg::{Grammar, Symbol},
    lr::{Event, Parse, ParseError, Precedence, ArrayParsingTable, Conflict},
};
use crate::cst::{CST, CSTBuilder};

pub struct ParserDef {
    pub lexer_def: LexerDef,
    pub var_names: Vec<String>,
    pub grammar: Grammar,
    pub token_precedence: Vec<Option<Precedence>>,
    pub production_precedence: Vec<Option<Precedence>>,
}

pub struct Parser {
    pub lexer: Lexer,
    pub var_names: Vec<String>,
    pub parsing_table: ArrayParsingTable,
}

impl ParserDef {
    #[must_use]
    pub fn new(lexer_def: LexerDef, var_names: Vec<String>, grammar: Grammar) -> Self {
        let word_count = grammar.max_word().map_or(0, |word| word + 1);
        Self {
            lexer_def,
            var_names,
            grammar,
            token_precedence: vec![None; word_count],
            production_precedence: Vec::new(),
        }
    }

    pub fn attach_precedence(&mut self, token_precedence: Vec<Option<Precedence>>) {
        self.token_precedence = token_precedence;
        
        // Production precedence is defaulted to the precedence of the rightmost token.
        // If this is None, then the production precedence is also None.
        self.production_precedence = self.grammar.alts().map(|alt| {
            let word = alt.iter().rev().find_map(|&symbol| {
                if let Symbol::Terminal(a) = symbol {
                    Some(a)
                } else {
                    None
                }
            });
            self.token_precedence[word?].clone()
        }).collect();
    }

    /// # Errors
    pub fn compile(&self) -> Result<Parser, Conflict> {
        Ok(Parser {
            lexer: self.lexer_def.compile(),
            var_names: self.var_names.to_vec(),
            parsing_table: ArrayParsingTable::with_precedence(&self.grammar, &self.token_precedence, &self.production_precedence)?,
        })
    }
}

impl<'a> Parser {
    /// # Errors
    #[must_use]
    pub fn parse<I>(&'a self, input: &'a I) -> Parse<ArrayParsingTable, Scan<'a, ArrayScanningTable, I>, Token<'a, I>, impl Fn(&Token<'a, I>) -> usize>
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
                Event::Shift(token) => builder.leaf(token),
                Event::Reduce { var, child_count, production: _ } => builder.branch(var, child_count),
            }
        }

        Ok(builder.build())
    }
}