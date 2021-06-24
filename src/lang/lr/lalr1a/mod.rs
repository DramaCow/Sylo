use std::collections::{HashSet, HashMap};
use crate::lang::{
    cfg::Grammar,
    lr::{LR0A, lr0a::State},
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

    // /// # Errors
    // pub fn dot<T, U>(&self, grammar: &Grammar, word_names: &[T], var_names: &[U], print_itemsets: bool) -> Result<String, std::fmt::Error>
    // where
    //     T: std::fmt::Display,
    //     U: std::fmt::Display,
    // {
    //     let word_to_string = |word: usize| {
    //         format!("{}", word_names[word])
    //     };

    //     let var_to_string = |var: usize| {
    //         if var < var_names.len() {
    //             format!("{}", var_names[var])
    //         } else {
    //             "S'".to_string()
    //         }
    //     };

    //     todo!()
    //     // graphviz::dot_with_labelling(grammar, self, word_to_string, var_to_string, print_itemsets)
    // }
}

mod builder;
pub use self::builder::{
    LALR1ABuilder,
    NonterminalTransition,
};

