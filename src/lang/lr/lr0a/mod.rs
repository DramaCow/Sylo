#![allow(non_snake_case)]

use std::collections::{BTreeSet, HashMap};
use crate::lang::cfg::{Grammar, Symbol};
use super::LR0Item;

pub struct LR0A {
    states: Vec<State>,
}

pub struct State {
    pub items: BTreeSet<LR0Item>,
    pub next: HashMap<Symbol, usize>,
}

impl LR0A {
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

        graphviz::dot_with_labelling(grammar, self, labelling)
    }
}

mod builder;
pub use builder::LR0ABuilder;

// =================
// === INTERNALS ===
// =================

mod graphviz;