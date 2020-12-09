use crate::lex::{LexAnalyzer, LexDef};
use crate::syn::{SynAnalyzer, SynDef, ConstructionError};

pub struct ParserDef {
    pub lex_def: LexDef,
    pub syn_def: SynDef,
}

pub struct Parser {
    pub(crate) lex: LexAnalyzer,
    pub(crate) syn: SynAnalyzer,
}

impl Parser {
    /// # Errors
    pub fn try_compile(def: &ParserDef) -> Result<Self, ConstructionError> {
        Ok(Self {
            lex: LexAnalyzer::compile(&def.lex_def),
            syn: SynAnalyzer::try_compile(&def.syn_def)?,
        })
    }
}