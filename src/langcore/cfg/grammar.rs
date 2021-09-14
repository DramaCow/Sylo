#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    Terminal(usize),
    Variable(usize), // aka. nonterminal
}

/// Barebones representation of a context free grammar.
#[derive(Clone)]
pub struct Grammar {
    lhs:     Vec<usize>,
    symbols: Vec<Symbol>, // symbols that occur in RHS of productions
    alts:    Vec<usize>,  // start index of each alt in symbols
    rules:   Vec<usize>,  // start index of each rule in alts
}

pub struct RuleView<'a> {
    grammar: &'a Grammar,
}

pub struct ProductionView<'a> {
    grammar: &'a Grammar,
}

pub struct Rules<'a> {
    view: RuleView<'a>,
    rule: usize,
}

pub struct Productions<'a> {
    view: ProductionView<'a>,
    production: usize,
}

pub struct Rule<'a> {
    grammar: &'a Grammar,
    alt_first: usize,
    alt_last: usize,
}

pub struct Alternatives<'a> {
    grammar: &'a Grammar,
    alt: usize,
    last_alt: usize,
}

impl Grammar {
    #[must_use]
    pub fn word_count(&self) -> usize {
        self.symbols.iter()
            .filter_map(|symbol| if let Symbol::Terminal(word) = symbol { Some(*word) } else { None })
            .max()
            .map_or(0, |word| word + 1)
    }

    #[must_use]
    pub fn rules(&self) -> RuleView {
        RuleView { grammar: self }
    }

    #[must_use]
    pub fn productions(&self) -> ProductionView {
        ProductionView { grammar: self }
    }
}

impl<'a> IntoIterator for RuleView<'a> {
    type Item = Rule<'a>;
    type IntoIter = Rules<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Rules {
            view: self.grammar.rules(),
            rule: 0,
        }
    }
}

impl<'a> RuleView<'a> {
    #[must_use]
    pub fn get(&self, index: usize) -> Rule<'a> {
        Rule {
            grammar: self.grammar,
            alt_first: self.grammar.rules[index],
            alt_last: self.grammar.rules[index + 1],
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.grammar.rules.len() - 1
    }
}

impl<'a> IntoIterator for ProductionView<'a> {
    type Item = (usize, &'a [Symbol]);
    type IntoIter = Productions<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            view: self.grammar.productions(),
            production: 0,
        }
    }
}

impl<'a> ProductionView<'a> {
    #[must_use]
    pub fn get(&self, index: usize) -> (usize, &'a [Symbol]) {
        let low = self.grammar.alts[index];
        let high = self.grammar.alts[index + 1];
        (self.grammar.lhs[index], &self.grammar.symbols[low..high])
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.grammar.lhs.len()
    }
}

impl<'a> Iterator for Rules<'a> {
    type Item = Rule<'a>;

    fn next(&mut self) -> Option<Self::Item> { 
        if self.rule < self.view.len() {
            let index = self.rule;
            self.rule += 1;
            Some(self.view.get(index))
        } else {
            None
        }
    }
}

impl<'a> Iterator for Productions<'a> {
    type Item = (usize, &'a [Symbol]);

    fn next(&mut self) -> Option<Self::Item> { 
        if self.production < self.view.len() {
            let index = self.production;
            self.production += 1;
            Some(self.view.get(index))
        } else {
            None
        }
    }
}

impl<'a> Rule<'a> {
    #[must_use]
    pub fn alts<'b>(&'b self) -> Alternatives<'a> {
        Alternatives {
            grammar: self.grammar,
            alt: self.alt_first,
            last_alt: self.alt_last,
        }
    }

    #[must_use]
    pub fn alt_indices(&self) -> std::ops::Range<usize> {
        self.alt_first..self.alt_last
    }
}

impl<'a> Iterator for Alternatives<'a> {
    type Item = &'a [Symbol];

    fn next(&mut self) -> Option<Self::Item> {
        if self.alt < self.last_alt {
            let low  = self.grammar.alts[self.alt];
            let high = self.grammar.alts[self.alt + 1];
            self.alt += 1;
            Some(&self.grammar.symbols[low..high])
        } else {
            None
        }
    }
}

pub struct GrammarBuilder {
    grammar: Grammar,
}

pub enum GrammarBuildError {
    InvalidVariable { rule: usize, alt: usize, pos: usize, variable: usize },
} 

// consuming builder
impl GrammarBuilder {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            grammar: Grammar {
                lhs: Vec::new(),
                symbols: Vec::new(),
                alts: vec![0],
                rules: vec![0],
            },
        }
    }

    /// # Panics
    #[must_use]
    pub fn rule(mut self, rule: &[&[Symbol]]) -> Self {
        let lhs = self.grammar.lhs.len();
        for &alt in rule {
            self.grammar.lhs.push(lhs);
            self.grammar.symbols.append(&mut alt.to_vec());
            self.grammar.alts.push(self.grammar.symbols.len());
        }
        self.grammar.rules.push(self.grammar.rules.last().unwrap() + rule.len());
        self
    }

    /// # Errors
    /// # Panics
    pub fn build(mut self) -> Result<Grammar, GrammarBuildError> {
        // Iterates through each rule and checks to see
        // if each variable is valid. If not, user receives 
        // error corresponding to the first erroneous symbol.
        for (i, rule) in self.grammar.rules().into_iter().enumerate() {
            for (j, alt) in rule.alts().enumerate() {
                for (k, symbol) in alt.iter().enumerate() {
                    if let Symbol::Variable(A) = symbol {
                        if *A >= self.grammar.rules().len() { 
                            return Err(GrammarBuildError::InvalidVariable {
                                rule: i,
                                alt: j,
                                pos: k,
                                variable: *A,
                            })
                        }
                    }
                }
            }
        }

        // finally, we augment the grammar by adding a start rule
        self.grammar.lhs.push(self.grammar.lhs.len());
        self.grammar.symbols.push(Symbol::Variable(0));
        self.grammar.alts.push(self.grammar.symbols.len());
        self.grammar.rules.push(self.grammar.rules.last().unwrap() + 1);

        Ok(self.grammar)
    }
}

impl std::fmt::Debug for GrammarBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidVariable { rule, alt, pos, variable } => {
                write!(f, "RHS of grammar rule {}:{}:{} refers to invalid variable ({})", rule, alt, pos, variable)
            },
        }
    }
}