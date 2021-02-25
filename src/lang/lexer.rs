use super::{
    Command,
    lex::{self, Parse},
};

pub struct LexerDef {
    pub labels: Vec<String>,
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
            labels: self.labels.to_vec(),
            lex: self.lex_def.compile(),
            commands: self.commands.to_vec(),
        }
    }
}

impl Lexer {
    #[must_use]
    pub fn scan<'a>(&'a self, text: &'a str) -> Parse<'a> {
        Parse::new(&self.lex, text)
    }
}