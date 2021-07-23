use crate::langcore::cfg::{Grammar};
use super::{inner, Action, NaiveLR1Table};

#[derive(Debug)]
pub struct ConstructionError {
    pub state: usize,
    pub conflict: Conflict,
}

#[derive(Debug)]
pub enum Conflict {
    ShiftReduce { word: usize, next_state: usize, production: usize },
    ReduceReduce { production1: usize, production2: usize },
}

pub trait LR1TableConstructionStrategy {
    /// # Errors
    fn construct<F: FnMut(Conflict) -> Result<Action, Conflict>>(grammar: &Grammar, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError>;
}

// impl<'a, T> LR1TableConstructionStrategy<'a> for T
// where
//     T: From<&'a Grammar> + inner::BuildLR1Table<'a>
// {
//     fn construct<F: FnMut(Conflict) -> Result<Action, Conflict>>(grammar: &'a Grammar, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError> {
//         let tmp = T::from(grammar);
//         inner::BuildLR1Table::build_lr1_table(&tmp, grammar, conflict_resolution)
//         // tmp.build_lr1_table(grammar, conflict_resolution)
//     }
// }