use super::{
    Command,
    lex::{self, Token},
    syn,
};
use crate::cst::{CST, CSTBuilder};

pub struct ParserDef {
    pub lex_labels: Vec<String>,
    pub syn_labels: Vec<String>,
    pub lex_def: lex::LexDef,
    pub syn_def: syn::SynDef,
    pub commands: Vec<Command>,
}

pub struct Parser {
    pub lex_labels: Vec<String>,
    pub syn_labels: Vec<String>,
    lex: lex::LexAnalyzer,
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
            lex_labels: self.lex_labels.to_vec(),
            syn_labels: self.syn_labels.to_vec(),
            lex: self.lex_def.compile(),
            syn: self.syn_def.compile()?,
            commands: self.commands.to_vec(),
        })
    }
}

impl Parser {
    /// # Errors
    pub fn tokenize<'a>(&'a self, text: &'a str) -> Result<Vec<Token>, lex::ParseError> {
        self.lex.parse(text).collect()
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