use super::{
    Vocabulary,
    re::{RegEx, Command, ArrayScanningTable, Scan},
};

pub struct LexerDef {
    pub vocab: Vocabulary,
    pub regexes: Vec<RegEx>,
    pub commands: Vec<Command>,
}

pub struct Lexer {
    vocab: Vocabulary,
    table: ArrayScanningTable,
}

impl LexerDef {
    #[must_use]
    pub fn compile(&self) -> Lexer {
        Lexer {
            vocab: self.vocab.clone(),
            table: ArrayScanningTable::new(&self.regexes, self.commands.iter().copied()),
        }
    }
}

impl<'a> Lexer {
    #[must_use]
    pub fn scan<I>(&'a self, input: &'a I) -> Scan<'a, ArrayScanningTable, I>
    where
        I: AsRef<[u8]> + ?Sized
    {
        Scan::new(&self.table, input)
    }
}