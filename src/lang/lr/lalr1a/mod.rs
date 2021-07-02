#![allow(non_snake_case)]

use std::collections::{HashSet, HashMap};
use crate::lang::{
    cfg::{Grammar, Symbol},
    lr::{LR0A, lr0a::State, LR0Item},
};

pub struct LALR1A {
    lr0a: LR0A,
    lookahead: HashMap<StateReductionPair, HashSet<Option<usize>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateReductionPair {
    pub state: usize,
    pub production: usize,
}

impl LALR1A {
    #[must_use]
    pub fn states(&self) -> &[State] {
        self.lr0a.states()
    }

    /// # Errors
    pub fn dot<T, U>(&self, grammar: &Grammar, word_names: &[T], var_names: &[U]) -> Result<String, std::fmt::Error>
    where
        T: std::fmt::Display,
        U: std::fmt::Display,
    {
        let labelling = |symbol: Symbol| {
            match symbol {
                Symbol::Terminal(a) => word_names[a].to_string(),
                Symbol::Variable(A) => if A < var_names.len() { var_names[A].to_string() } else { "S'".to_string() },
            }
        };

        LALR1ADotWriter::new(String::new(), self, grammar, labelling).build()
    }
}

mod builder;
pub use self::builder::{
    LALR1ABuilder,
    NonterminalTransition,
};

mod graphviz;
pub use self::graphviz::LALR1ADotWriter;