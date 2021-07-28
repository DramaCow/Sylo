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

// pub trait LR1TableConstructionStrategy {
//     type Builder;

//     fn new_builder(grammar: &Grammar) -> Self::Builder;

//     /// # Errors
//     fn build<F: FnMut(Conflict) -> Result<Action, Conflict>>(builder: &Self::Builder, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError>;

//     /// # Errors
//     fn construct<F: FnMut(Conflict) -> Result<Action, Conflict>>(grammar: &Grammar, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError> {
//         Self::build(&Self::new_builder(grammar), conflict_resolution)
//     }
// }

impl<T> LR1TableConstructionStrategy for T
where
    for<'a> T: From<&'a Grammar> + inner::BuildLR1Table<'a>
{
    fn construct<F: FnMut(Conflict) -> Result<Action, Conflict>>(grammar: &Grammar, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError> {
        T::from(grammar).build_lr1_table(grammar, conflict_resolution)
    }
}