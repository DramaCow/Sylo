use super::Vocabulary;
use crate::lang::re::{RegEx, Command, LexTable, Scan};

pub struct LexerDef {
    pub vocab: Vocabulary,
    pub regexes: Vec<RegEx>,
    pub commands: Vec<Command>,
}

pub struct Lexer {
    pub(super) vocab: Vocabulary,
    pub(super) table: LexTable,
}

impl LexerDef {
    #[must_use]
    pub fn compile(&self) -> Lexer {
        Lexer {
            vocab: self.vocab.clone(),
            table: LexTable::new(&self.regexes, self.commands.iter().copied()),
        }
    }
}

impl<'a> Lexer {
    #[must_use]
    pub fn scan<I>(&'a self, input: &'a I) -> Scan<'a, LexTable, I>
    where
        I: AsRef<[u8]> + ?Sized
    {
        Scan::new(&self.table, input)
    }
}