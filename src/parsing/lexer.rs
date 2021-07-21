use crate::langcore::re::{self, RegEx};

pub struct LexerDefBuilder {
    def: LexerDef,
}

pub struct LexerDef {
    pub vocab: Vec<String>,
    pub regexes: Vec<re::RegEx>,
    pub commands: Vec<re::Command>,
}

impl LexerDefBuilder {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { def: LexerDef { vocab: Vec::new(), regexes: Vec::new(), commands: Vec::new() } }
    }

    pub fn rule(&mut self, label: String, regex: RegEx) -> &mut Self {
        self._rule_interal(label, regex, re::Command::Emit)
    }

    pub fn skip(&mut self, label: String, regex: RegEx) -> &mut Self {
        self._rule_interal(label, regex, re::Command::Skip)
    }

    #[must_use]
    pub fn vocab(&self) -> &[String] {
        &self.def.vocab
    }

    /// # Panics
    #[must_use]
    pub fn build(self) -> LexerDef {
        if self.def.regexes.is_empty() {
            panic!("Need at least 1 RegEx.")
        }
        self.def
    }

    fn _rule_interal(&mut self, label: String, regex: RegEx, command: re::Command) -> &mut Self {
        self.def.vocab.push(label);
        self.def.regexes.push(regex);
        self.def.commands.push(command);
        self
    }
}

impl LexerDef {
    /// # Panics
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
}

pub struct Lexer {
    pub vocab: Vec<String>,
    pub(super) table: re::NaiveLexTable,
}

pub type Scan<'a> = re::Scan<'a, re::NaiveLexTable>;

impl<'a> Lexer {
    pub fn scan<I: AsRef<[u8]> + ?Sized>(&'a self, input: &'a I) -> Scan<'a> {
        Scan::new(&self.table, input)
    }
}