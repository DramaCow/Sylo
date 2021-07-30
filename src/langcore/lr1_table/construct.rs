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

// pub trait Strategy<T> {}
pub trait LR1TableConstructionStrategy<'a> {
    type Builder;

    fn builder(&self, grammar: &'a Grammar) -> Self::Builder;

    /// # Errors 
    fn build<F>(builder: &Self::Builder, grammar: &Grammar, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError>
    where
        F: FnMut(Conflict) -> Result<Action, Conflict>;
}

impl<'a, T> LR1TableConstructionStrategy<'a> for T
where
    T: inner::InnerStrategy<'a>,
    T::Builder: for<'b> inner::BuildLR1Table<'b>,
{
    type Builder = T::Builder;

    fn builder(&self, grammar: &'a Grammar) -> Self::Builder {
        self.builder(grammar)
    }

    /// # Errors 
    fn build<F>(builder: &Self::Builder, grammar: &Grammar, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError>
    where
        F: FnMut(Conflict) -> Result<Action, Conflict>
    {
        <Self::Builder as inner::BuildLR1Table>::build_lr1_table(builder, grammar, conflict_resolution)
    }
}