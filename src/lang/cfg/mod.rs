//! Context free grammar.

#![allow(non_snake_case)]

mod first;
pub use self::first::First;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    Terminal(usize),
    Variable(usize), // aka. nonterminal
}

/// Barebones representation of a context free grammar.
pub struct Grammar {
    symbols:  Vec<Symbol>,
    alts:     Vec<usize>,  // start index of each alt in symbols
    rules:    Vec<usize>,  // start index of each rule in alts
}

pub struct Rules<'a> {
    grammar: &'a Grammar,
    rule: usize,
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
    pub fn var_count(&self) -> usize {
        self.rules.len() - 1
    }

    #[must_use]
    pub fn alt_count(&self) -> usize {
        self.alts.len() - 1
    }

    #[must_use]
    pub fn rules(&self) -> Rules {
        Rules {
            grammar: self,
            rule: 0,
        }
    }

    #[must_use]
    pub fn rule(&self, A: usize) -> Rule {
        let low  = self.rules[A];
        let high = self.rules[A + 1];
        Rule {
            grammar: self,
            alt_first: low,
            alt_last: high,
        }
    }

    #[must_use]
    pub fn alts(&self) -> Alternatives {
        Alternatives {
            grammar: self,
            alt: 0,
            last_alt: self.alt_count(),
        }
    }

    #[must_use]
    pub fn alt(&self, i: usize) -> &[Symbol] {
        let low  = self.alts[i];
        let high = self.alts[i + 1];
        &self.symbols[low..high]
    }

    #[must_use]
    pub fn max_word(&self) -> Option<usize> {
        self.symbols.iter().filter_map(|symbol| if let Symbol::Terminal(word) = symbol { Some(*word) } else { None }).max()
    }
}

impl<'a> Iterator for Rules<'a> {
    type Item = Rule<'a>;

    fn next(&mut self) -> Option<Self::Item> { 
        if self.rule < self.grammar.rules.len() - 1 {
            let index = self.rule;
            self.rule += 1;
            Some(self.grammar.rule(index))
        } else {
            None
        }
    }
}

impl Rule<'_> {
    #[must_use]
    pub fn alts(&self) -> Alternatives {
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
                symbols: Vec::new(),
                alts: vec![0],
                rules: vec![0],
            },
        }
    }

    #[must_use]
    pub fn rule(mut self, rule: &[&[Symbol]]) -> Self {
        self.grammar.rules.push(self.grammar.rules.last().unwrap() + rule.len());
        for &alt in rule {
            self.grammar.symbols.append(&mut alt.to_vec());
            self.grammar.alts.push(self.grammar.symbols.len());
        }
        self
    }

    /// # Errors
    pub fn try_build(mut self) -> Result<Grammar, GrammarBuildError> {
        // Iterates through each rule and checks to see
        // if each variable is valid. If not, user receives 
        // error corresponding to the first erroneous symbol.
        for (i, rule) in self.grammar.rules().enumerate() {
            for (j, alt) in rule.alts().enumerate() {
                for (k, symbol) in alt.iter().enumerate() {
                    if let Symbol::Variable(A) = symbol {
                        if *A >= self.grammar.var_count() { 
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

        // finally, we add a start rule
        self.grammar.rules.push(self.grammar.rules.last().unwrap() + 1);
        self.grammar.symbols.push(Symbol::Variable(0));
        self.grammar.alts.push(self.grammar.symbols.len());

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