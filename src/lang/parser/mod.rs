use crate::lang::lex::{self, Token};
use crate::lang::syn;
use crate::cst::{CST, CSTBuilder};

pub use self::compile::ParserDef;

#[derive(Clone)]
pub enum Command {
    Skip,
    Emit,
}

pub struct Parser {
    pub lex_labels: Vec<String>,
    pub syn_labels: Vec<String>,
    lex: lex::LexAnalyzer,
    syn: syn::SynAnalyzer,
    commands: Vec<Command>
}

#[derive(Debug)]
pub enum ParseError<'a> {
    Lex(lex::ParseError<'a>),
    Syn(syn::ParseError),
}

impl Parser {
    /// # Errors
    pub fn tokenize<'a>(&'a self, text: &'a str) -> Result<Vec<Token>, lex::ParseError> {
        self.lex.parse(text).collect()
    }

    /// # Errors
    pub fn cst<'a>(&'a self, text: &'a str) -> Result<CST, ParseError> {
        match self.tokenize(text) {
            Ok(tokens) => {
                let mut builder = CSTBuilder::new();
        
                for res in self.syn.parse(tokens.iter().map(|token| token.class)) {
                    match res {
                        Ok(step) => {
                            match step {
                                syn::Instruction::Shift { word: _, index } => builder.leaf(index),
                                syn::Instruction::Reduce { var, count } => {
                                    match self.commands.get(var).unwrap() {
                                        Command::Emit => builder.branch(var, count),
                                        Command::Skip => builder.list(count),
                                    };
                                },
                            }
                        },
                        Err(error) => {
                            return Err(ParseError::Syn(error));
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

// =================
// === INTERNALS ===
// =================

mod compile;