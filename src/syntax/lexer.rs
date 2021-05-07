use crate::lang::re::{RegEx, Command, Token, NaiveLexTable, Scan, ScanError};

#[derive(Default)]
pub struct LexerBuilder {
    vocab: Vec<String>,
    regexes: Vec<RegEx>,
    commands: Vec<Command>,
}

pub struct Lexer {
    pub(super) vocab: Vec<String>,
    pub(super) table: NaiveLexTable,
}

impl LexerBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self { vocab: Vec::new(), regexes: Vec::new(), commands: Vec::new() }
    }

    pub fn rule(&mut self, label: String, regex: RegEx) -> &mut Self {
        self._rule_interal(label, regex, Command::Emit)
    }

    pub fn skip(&mut self, label: String, regex: RegEx) -> &mut Self {
        self._rule_interal(label, regex, Command::Skip)
    }

    #[must_use]
    pub fn vocab(&self) -> &[String] {
        &self.vocab
    }

    fn _rule_interal(&mut self, label: String, regex: RegEx, command: Command) -> &mut Self {
        self.vocab.push(label);
        self.regexes.push(regex);
        self.commands.push(command);
        self
    }

    #[must_use]
    pub fn compile(&self) -> Lexer {
        if self.regexes.is_empty() {
            panic!()
        }
        
        Lexer {
            vocab: self.vocab.clone(),
            table: NaiveLexTable::new(&self.regexes, self.commands.iter().copied()),
        }
    }
}

impl<'a> Lexer {
    // pub fn scan<I>(&'a self, input: &'a I) -> Scan<'a, LexTable, I>
    pub fn scan<I>(&'a self, input: &'a I) -> impl Iterator<Item = Result<Token<'a, I>, ScanError>>
    where
        I: AsRef<[u8]> + ?Sized
    {
        Scan::new(&self.table, input)
    }
}