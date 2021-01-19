use crate::lang::{lex, syn};
use super::{Command, Parser};

pub struct ParserDef {
    pub lex_def: lex::LexAnalyzerDef,
    pub syn_def: syn::SynAnalyzerDef,
    pub commands: Vec<Command>,
}

impl ParserDef {
    /// # Errors
    pub fn compile(&self) -> Result<Parser, syn::CompileError> {
        Ok(Parser {
            lex_labels: self.lex_def.labels.to_vec(),
            syn_labels: self.syn_def.labels.to_vec(),
            lex: self.lex_def.compile(),
            syn: self.syn_def.compile()?,
            commands: self.commands.to_vec(),
        })
    }
}