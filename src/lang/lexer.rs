use super::{
    Vocabulary,
    Command,
    lex::{self, Parse},
};

pub struct LexerDef {
    pub vocab: Vocabulary,
    pub lex_def: lex::LexDef,
    pub commands: Vec<Command>,
}

pub struct Lexer {
    vocab: Vocabulary,
    lex: lex::LexAnalyzer,
    commands: Vec<Command>
}

impl LexerDef {
    #[must_use]
    pub fn compile(&self) -> Lexer {
        Lexer {
            vocab: self.vocab.clone(),
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