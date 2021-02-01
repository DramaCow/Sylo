use crate::lang::{lex, syn};
use super::{Command, Parser};
use crate::lang::cfg::lr1::DFA;

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

    #[must_use]
    pub fn dot_lr1_dfa(&self) -> String {
        let dfa = DFA::from(&self.syn_def.grammar);
        dfa.dot(&self.syn_def.grammar, &self.lex_def.labels, &self.syn_def.labels)
    }
}