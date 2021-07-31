use crate::langcore::{re, cfg::{Grammar, Symbol}, lr1_table};
use crate::{lexer, cst};
use crate::langcore::lr1_table::LR1TableConstruction;

pub mod strategy {
    use crate::langcore::lr1_table::{self, strategy};
    pub use strategy::LALR1;
    pub use strategy::LR1;
}

pub struct ParserDefBuilder {
    def: ParserDef,
}

pub struct ParserDef {
    pub lexer_def: lexer::LexerDef,
    pub var_names: Vec<String>,
    pub grammar: Grammar,
    pub token_precedence: Vec<Option<Precedence>>,
    pub production_precedence: Vec<Option<Precedence>>,
}

#[derive(Debug, Clone)]
pub struct Precedence {
    pub level: usize,
    pub associativity: Associativity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Associativity {
    Left,
    Right,
    Nonassoc,
}

impl ParserDefBuilder {
    #[must_use]
    pub fn new(lexer_def: lexer::LexerDef, var_names: Vec<String>, grammar: Grammar) -> Self {
        let word_count = grammar.word_count();
        let production_count = grammar.production_count();

        Self {
            def: ParserDef {
                lexer_def,
                var_names,
                grammar,
                token_precedence: vec![None; word_count],
                production_precedence: vec![None; production_count],
            },
        }
    }

    pub fn set_token_precedence(&mut self, word: usize, precedence: Precedence) -> &mut Self {
        self.def.token_precedence[word] = Some(precedence);
        self
    }

    pub fn set_production_precedence(&mut self, production: usize, precedence: Precedence) -> &mut Self {
        self.def.production_precedence[production] = Some(precedence);
        self
    }

    #[must_use]
    pub fn build(mut self) -> ParserDef {
        // In case where no production precedence has been specified, production precedence
        // is defaulted to the precedence of the rightmost token (that has some precedence).
        for (i, alt) in self.def.grammar.rules().flat_map(|rule| rule.alts()).enumerate() {
            if self.def.production_precedence[i].is_none() {
                self.def.production_precedence[i] = alt.iter().rev().find_map(|&symbol| {
                    if let Symbol::Terminal(a) = symbol {
                        self.def.token_precedence[a].clone()
                    } else {
                        None
                    }
                });
            }
        }
        self.def
    }
}

impl ParserDef {
    pub fn conflict_resolution(&self) -> impl FnMut(lr1_table::Conflict) -> Result<lr1_table::Action, lr1_table::Conflict> + '_ {
        move |conflict| {
            match conflict {
                lr1_table::Conflict::ShiftReduce { word, next_state, production } => {
                    let tok  = if let Some(tok) = self.token_precedence[word].as_ref() {
                        tok
                    } else {
                        return Ok(lr1_table::Action::Shift(next_state))
                    };

                    let prod = if let Some(prod) = self.production_precedence[production].as_ref() {
                        prod
                    } else {
                        return Ok(lr1_table::Action::Shift(next_state))
                    };

                    if prod.level > tok.level || (prod.level == tok.level && prod.associativity == Associativity::Left) {
                        Ok(lr1_table::Action::Reduce(production))
                    } else {
                        Ok(lr1_table::Action::Shift(next_state))
                    }
                }
                lr1_table::Conflict::ReduceReduce { .. } => {
                    Err(conflict)
                }
            }
        }
    }

    /// # Errors
    pub fn build<S: LR1TableConstruction>(&self, strategy: &S) -> Result<Parser, lr1_table::ConstructionError> {
        Ok(Parser {
            lexer: self.lexer_def.build(),
            var_names: self.var_names.clone(),
            parsing_table: strategy.construct(&self.grammar, self.conflict_resolution())?,
        })
    }
}

pub struct Parser {
    pub lexer: lexer::Lexer,
    pub var_names: Vec<String>,
    pub parsing_table: lr1_table::NaiveLR1Table,
}

type Parse<'a, F> = lr1_table::Parse<'a, lr1_table::NaiveLR1Table, lexer::Scan<'a>, re::Token, F>;
type ParseError = lr1_table::ParseError<re::ScanError>;

impl<'a> Parser {
    /// # Errors
    pub fn parse<I: AsRef<[u8]> + ?Sized>(&'a self, input: &'a I) -> Parse<'a, impl Fn(&re::Token) -> usize> {
        Parse::new(&self.parsing_table, self.lexer.scan(input), |token: &re::Token| token.class)
    }

    /// # Errors
    pub fn cst(&'a self, text: &'a str) -> Result<cst::CST, ParseError> {
        let mut builder = cst::CSTBuilder::new();

        for res in self.parse(text) {
            match res? {
                lr1_table::Event::Shift(token) => builder.leaf(token.class, &text[token.span]),
                lr1_table::Event::Reduce { var, child_count, production: _ } => builder.branch(var, child_count),
            }
        }

        Ok(builder.build())
    }
}