use super::{Action, Reduction, LR1Table};

#[derive(Debug)]
pub struct ConstructionError {
    state: usize,
    conflict: Conflict,
}

#[derive(Debug)]
pub enum Conflict {
    ShiftReduce { word: usize, next_state: usize, production: usize },
    ReduceReduce { production1: usize, production2: usize },
}

#[derive(Debug)]
pub struct NaiveLR1Table {
    actions:    Vec<Action>,        /// lookup what action to perform given state and word
    gotos:      Vec<Option<usize>>, /// lookup what state should be transitioned to after reduction
    reductions: Vec<Reduction>,     // production --> rule and number of symbols
    word_count: usize,
    var_count:  usize,
}

impl LR1Table for NaiveLR1Table {
    fn action(&self, state: usize, word: Option<usize>) -> Action {
        self.actions[state * self.word_count + word.map_or(0, |a| a + 1)]
    }

    fn goto(&self, state: usize, var: usize) -> Option<usize> {
        self.gotos[state * self.var_count + var]
    }

    fn reduction(&self, alt: usize) -> Reduction {
        self.reductions[alt]
    }
}