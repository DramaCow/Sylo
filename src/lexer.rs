use regex_deriv as re;

pub struct LexerBuilder {
    vocab: Vec<String>,
    regexes: Vec<re::RegEx>,
    classes: Vec<Option<usize>>,
    num_classes: usize,
}

impl LexerBuilder {
    #[must_use]
    pub fn rule(mut self, label: String, pattern: re::RegEx) -> Self {
        self.vocab.push(label);
        self.regexes.push(pattern);
        self.classes.push(Some(self.num_classes));
        self.num_classes += 1;
        self
    }

    #[must_use]
    pub fn ignore(mut self, pattern: re::RegEx) -> Self {
        self.regexes.push(pattern);
        self.classes.push(None);
        self
    }

    /// # Panics
    #[must_use]
    pub fn build(self) -> Lexer {
        if self.regexes.is_empty() {
            panic!("Need at least 1 RegEx.")
        }
        
        Lexer {
            vocab: self.vocab,
            table: re::NaiveLexTable::new(&re::DFA::from(&self.regexes).minimize()),
            classes: self.classes,
        }
    }
}

pub struct Lexer {
    vocab: Vec<String>,
    table: re::NaiveLexTable,
    classes: Vec<Option<usize>>,
}

pub struct Scan<'a> {
    lexer: &'a Lexer,
    scan_imp: re::Scan<'a, re::NaiveLexTable>,
}

impl<'a> Iterator for Scan<'a> {
    type Item = Result<re::Token, re::ScanError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.scan_imp.next()? {
            Ok(token) => Some(Ok(token)),
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'a> Lexer {
    pub fn scan<I: AsRef<[u8]> + ?Sized>(&'a self, input: &'a I) -> Scan<'a> {
        Scan {
            lexer: self,
            scan_imp: re::Scan::new(&self.table, input),
        }
    }
}