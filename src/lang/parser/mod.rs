use crate::lang::lex;
use crate::lang::syn;
use cst::{CST, CSTBuilder};

pub mod cst;
pub mod parse;

pub struct ParserDef {
    pub lex_def: lex::LexAnalyzerDef,
    pub syn_def: syn::SynAnalyzerDef,
}

pub struct Parser {
    pub(crate) lex: lex::LexAnalyzer,
    pub(crate) syn: syn::SynAnalyzer,
}

impl Parser {
    /// # Errors
    pub fn try_compile(def: &ParserDef) -> Result<Self, syn::ConstructionError> {
        Ok(Self {
            lex: lex::LexAnalyzer::compile(&def.lex_def),
            syn: syn::SynAnalyzer::try_compile(&def.syn_def)?,
        })
    }

    /// # Errors
    pub fn cst<'a>(&'a self, text: &'a str) -> Result<CST, syn::SynParseError> {
        let tokens = self.lex.parse(text).collect::<Result<Vec<_>, _>>().unwrap();
        let tokens2 = self.lex.parse(text).map(|res| res.and_then(|token| Ok(token.class)));

        let mut builder = CSTBuilder::new();

        for step in self.syn.parse(tokens.iter().map(|token| token.class)) {
            match step? {
                syn::Action::Shift { word, index } => builder.leaf(word, index),
                syn::Action::Reduce { var, count } => {
                    match self.syn.commands.get(var).unwrap() {
                        syn::SynCommand::Emit => builder.branch(var, count),
                        syn::SynCommand::Skip => builder.list(count),
                    };
                },
            }
        }

        Ok(builder.build())
    }
}