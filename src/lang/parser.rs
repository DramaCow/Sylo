use super::{
    Command,
    lex::{self, Token},
    LexerDef,
    Lexer,
    syn,
};
use crate::cst::{CST, CSTBuilder};

pub struct ParserDef {
    pub lexer_def: LexerDef,
    pub var_names: Vec<String>,
    pub syn_def: syn::SynDef,
    pub commands: Vec<Command>,
}

pub struct Parser {
    pub lexer: Lexer,
    pub var_names: Vec<String>,
    pub syn: syn::SynAnalyzer,
    commands: Vec<Command>
}

#[derive(Debug)]
pub enum ParseError<'a> {
    Lex(lex::ParseError),
    Syn(Vec<Token<'a>>, syn::ParseError),
}

impl ParserDef {
    /// # Errors
    pub fn compile(&self) -> Result<Parser, syn::CompileError> {
        Ok(Parser {
            lexer: self.lexer_def.compile(),
            var_names: self.var_names.to_vec(),
            syn: self.syn_def.compile()?,
            commands: self.commands.to_vec(),
        })
    }
}

impl Parser {
    /// # Errors
    pub fn tokenize<'a>(&'a self, text: &'a str) -> Result<Vec<Token>, lex::ParseError> {
        self.lexer.scan(text).collect()
    }

    /// # Errors
    pub fn cst<'a>(&'a self, text: &'a str) -> Result<CST, ParseError> {
        // let iter = self.lex.parse(text).map(|res| Ok((res?.class, res?.lexeme)));
        // let tokenize = iter.unzip::<Result<(Vec<usize>, Vec<&str>), lex::ParseError>>();
        match self.tokenize(text) {
            Ok(tokens) => {
                let mut builder = CSTBuilder::new();
        
                for res in self.syn.parse(tokens.iter().map(|token| token.class)) {
                    match res {
                        Ok(step) => {
                            match step {
                                syn::Node::Word { word, index } => builder.leaf(word, index),
                                syn::Node::Var { var, child_count } => {
                                    match self.commands.get(var).unwrap() {
                                        Command::Emit => builder.branch(var, child_count),
                                        Command::Skip => builder.list(child_count),
                                    };
                                },
                            }
                        },
                        Err(error) => {
                            return Err(ParseError::Syn(tokens, error));
                        }
                    }
                }
        
                Ok(builder.build(tokens))
            },
            Err(error) => {
                Err(ParseError::Lex(error))
            }
        }
    }
}