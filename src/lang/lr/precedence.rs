use crate::lang::cfg::{Grammar, Symbol};

/// Wrapper class.
pub struct PrecedenceGrammar {
    grammar: Grammar,
    token_precedence: Vec<Option<Precedence>>,
    production_precedence: Vec<Option<Precedence>>,
}

pub struct PrecedenceGrammarBuilder(PrecedenceGrammar);

impl PrecedenceGrammarBuilder {
    #[must_use]
    pub fn new(grammar: Grammar) -> Self {
        let word_count = grammar.max_word().map_or(0, |word| word + 1);
        let production_count = grammar.alt_count();
        
        Self(PrecedenceGrammar {
            grammar,
            token_precedence: vec![None; word_count],
            production_precedence: vec![None; production_count],
        })
    }

    pub fn set_token_precedence(&mut self, word: usize, precedence: Precedence) -> &mut Self {
        self.0.token_precedence[word] = Some(precedence);
        self
    }

    pub fn set_production_precedence(&mut self, production: usize, precedence: Precedence) -> &mut Self {
        self.0.production_precedence[production] = Some(precedence);
        self
    }

    #[must_use]
    pub fn build(mut self) -> PrecedenceGrammar {
        // In case where no production precedence has been specified, production precedence
        // is defaulted to the precedence of the rightmost token (that has some precedence).
        for (i, alt) in self.0.grammar.alts().enumerate() {
            if self.0.production_precedence[i].is_none() {
                self.0.production_precedence[i] = alt.iter().rev().find_map(|&symbol| {
                    if let Symbol::Terminal(a) = symbol {
                        self.0.token_precedence[a].clone()
                    } else {
                        None
                    }
                });
            }
        }
        
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Precedence {
    level: usize,
    associativity: Associativity,
}

#[derive(Debug, Clone)]
pub enum Associativity {
    Left,
    Right,
    Nonassoc,
}