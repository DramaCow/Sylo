use crate::lang::re::{self, RegEx};

#[derive(Default)]
pub struct LexerBuilder {
    vocab: Vec<String>,
    regexes: Vec<re::RegEx>,
    commands: Vec<re::Command>,
}

pub struct Lexer {
    pub vocab: Vec<String>,
    pub(super) table: re::NaiveLexTable,
}

pub type Scan<'a, I> = re::Scan<'a, re::NaiveLexTable, I>;

impl LexerBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self { vocab: Vec::new(), regexes: Vec::new(), commands: Vec::new() }
    }

    pub fn rule(&mut self, label: String, regex: RegEx) -> &mut Self {
        self._rule_interal(label, regex, re::Command::Emit)
    }

    pub fn skip(&mut self, label: String, regex: RegEx) -> &mut Self {
        self._rule_interal(label, regex, re::Command::Skip)
    }

    #[must_use]
    pub fn vocab(&self) -> &[String] {
        &self.vocab
    }

    #[must_use]
    pub fn build(&self) -> Lexer {
        if self.regexes.is_empty() {
            panic!("Need at least 1 RegEx.")
        }
        
        Lexer {
            vocab: self.vocab.clone(),
            table: re::NaiveLexTable::new(&self.regexes, self.commands.iter().copied()),
        }
    }

    fn _rule_interal(&mut self, label: String, regex: RegEx, command: re::Command) -> &mut Self {
        self.vocab.push(label);
        self.regexes.push(regex);
        self.commands.push(command);
        self
    }
}

impl<'a> Lexer {
    pub fn scan<I>(&'a self, input: &'a I) -> Scan<'a, I>
    where
        I: AsRef<[u8]> + ?Sized
    {
        Scan::new(&self.table, input)
    }
}