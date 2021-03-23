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
    pub syn: lr1::UncompressedTable,
    commands: Vec<Command>
}

impl ParserDef {
    /// # Errors
    pub fn compile(&self) -> Result<Parser, lr1::ConstructionError> {
        Ok(Parser {
            lexer: self.lexer_def.compile(),
            var_names: self.var_names.to_vec(),
            syn: lr1::UncompressedTable::new(&self.grammar)?,
            commands: self.commands.to_vec(),
        })
    }
}

impl<'a> Parser {
    /// # Errors
    pub fn tokenize(&'a self, text: &'a str) -> Result<Vec<Token>, ScanError> {
        self.lexer.scan(text).collect()
    }

    /// # Errors
    #[must_use]
    pub fn parse(&'a self, text: &'a str) -> Parse<lr1::UncompressedTable, Scan<'a>, Token, impl Fn(&Token<'a>) -> usize> {
        Parse::new(&self.syn, self.lexer.scan(text), |token: &Token| token.class)
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