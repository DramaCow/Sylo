use crate::langcore::cfg::Grammar;
use super::{Action, NaiveLR1Table};

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
    fn construct<F: FnMut(Conflict) -> Result<Action, Conflict>>(self, grammar: &Grammar, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError>;
}

// helper functions

/// # Errors
pub fn with_conflict_resolution<S, F>(grammar: &Grammar, strategy: S, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError>
where
    S: LR1TableConstructionStrategy,
    F: FnMut(Conflict) -> Result<Action, Conflict>
{
    S::construct(strategy, grammar, conflict_resolution)
}

/// # Errors
pub fn construct<S>(grammar: &Grammar, strategy: S) -> Result<NaiveLR1Table, ConstructionError>
where
    S: LR1TableConstructionStrategy,
{
    S::construct(strategy, grammar, |conflict: Conflict| { Err(conflict) })
}