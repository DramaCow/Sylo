use crate::lang::{lex, syn};
use super::{Command, Parser};
use crate::lang::cfg::lr1::LR1A;

pub struct ParserDef {
    pub lex_labels: Vec<String>,
    pub syn_labels: Vec<String>,
    pub lex_def: lex::LexDef,
    pub syn_def: syn::SynDef,
    pub commands: Vec<Command>,
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

    #[must_use]
    pub fn dot_lr1a(&self) -> String {
        let lr1a = LR1A::from(&self.syn_def.grammar);
        lr1a.dot(&self.syn_def.grammar, &self.lex_labels, &self.syn_labels)
    }
}