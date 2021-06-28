#![allow(non_snake_case)]

use std::collections::{BTreeSet, HashMap};
use crate::lang::cfg::{Grammar, Symbol};
use super::{LRkItem, LR1Item};

pub struct LR1A {
    states: Vec<State>,
}

pub struct State {
    pub items: BTreeSet<LR1Item>,
    pub next: HashMap<Symbol, usize>,
}

impl LR1A {
    #[must_use]
    pub fn states(&self) -> &[State] {
        &self.states
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

        LR1ADotWriter::new(String::new(), self, grammar, labelling).build()
    }
}

mod builder;
pub use builder::LR1ABuilder;

mod graphviz;
pub use graphviz::LR1ADotWriter;