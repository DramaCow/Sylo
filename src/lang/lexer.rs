use super::{
    Command,
    lex::{self, Token},
};

pub struct LexerDef {
    pub lex_labels: Vec<String>,
    pub lex_def: lex::LexDef,
    pub commands: Vec<Command>,
}

pub struct Lexer {
    labels: Vec<String>,
    lex: lex::LexAnalyzer,
    commands: Vec<Command>
}

impl LexerDef {
    #[must_use]
    pub fn compile(&self) -> Lexer {
        Lexer {
            labels: self.lex_labels.to_vec(),
            lex: self.lex_def.compile(),
            commands: self.commands.to_vec(),
        }
    }
}