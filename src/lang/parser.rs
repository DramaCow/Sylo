use super::{
    Command,
    lex::{Token, Scan, ScanError},
    LexerDef,
    Lexer,
    cfg::Grammar,
    lr::{ParseTreeNode, Parse, ParseError},
    lr1,
};
use crate::cst::{CST, CSTBuilder};

pub struct ParserDef {
    pub lexer_def: LexerDef,
    pub var_names: Vec<String>,
    pub grammar: Grammar,
    pub commands: Vec<Command>,
}

pub struct Parser {
    pub lexer: Lexer,
    pub var_names: Vec<String>,
    pub syn: lr1::ArrayParsingTable,
    commands: Vec<Command>
}

impl ParserDef {
    /// # Errors
    pub fn compile(&self) -> Result<Parser, lr1::ConstructionError> {
        Ok(Parser {
            lexer: self.lexer_def.compile(),
            var_names: self.var_names.to_vec(),
            syn: lr1::ArrayParsingTable::new(&self.grammar)?,
            commands: self.commands.to_vec(),
        })
    }
}

impl<'a> Parser {
    /// # Errors
    #[must_use]
    pub fn parse<I>(&'a self, input: &'a I) -> Parse<lr1::ArrayParsingTable, Scan<'a, I>, Token<'a, I>, impl Fn(&Token<'a, I>) -> usize>
    where
        I: AsRef<[u8]> + ?Sized
    {
        Parse::new(&self.syn, self.lexer.scan(input), |token: &Token<'a, I>| token.class)
    }

    /// # Errors
    pub fn cst(&'a self, text: &'a str) -> Result<CST, ParseError<ScanError>> {
        let mut builder = CSTBuilder::new();

        for res in self.parse(text) {
            match res? {
                ParseTreeNode::Word(token) => builder.leaf(token),
                ParseTreeNode::Var { var, child_count } => {
                    match self.commands[var] {
                        Command::Emit => builder.branch(var, child_count),
                        Command::Skip => builder.list(child_count),
                    };
                },
            }
        }

        Ok(builder.build())
    }
}