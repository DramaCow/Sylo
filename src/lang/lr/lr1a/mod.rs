#![allow(non_snake_case)]

use std::collections::{BTreeSet, HashMap};
use crate::lang::cfg::{Grammar, Symbol};
use super::LR1Item;

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
    pub fn dot<T, U>(&self, grammar: &Grammar, word_names: &[T], var_names: &[U], print_itemsets: bool) -> Result<String, std::fmt::Error>
    where
        T: std::fmt::Display,
        U: std::fmt::Display,
    {
        let word_to_string = |word: usize| {
            format!("{}", word_names[word])
        };

        let var_to_string = |var: usize| {
            if var < var_names.len() {
                format!("{}", var_names[var])
            } else {
                "S'".to_string()
            }
        };

        graphviz::dot_with_labelling(grammar, self, word_to_string, var_to_string, print_itemsets)
    }
}

mod builder;
pub use builder::LR1ABuilder;

// =================
// === INTERNALS ===
// =================

mod graphviz;