use crate::langcore::cfg::{Grammar, Symbol};
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

/// Most generic definition
pub trait LR1TableConstruction {
    /// # Errors 
    fn construct<F>(&self, grammar: &Grammar, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError>
    where
        F: FnMut(Conflict) -> Result<Action, Conflict>;
}

impl<T> LR1TableConstruction for T
where
    T: for<'b> LR1TableBuilderStrategy<'b>,
{
    /// # Errors 
    fn construct<F>(&self, grammar: &Grammar, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError>
    where
        F: FnMut(Conflict) -> Result<Action, Conflict>
    {
        T::build(&self.builder(grammar), grammar, conflict_resolution)
    }
}

pub trait LR1TableBuilderStrategy<'a> {
    type Builder;

    fn builder(&self, grammar: &'a Grammar) -> Self::Builder;

    /// # Errors 
    fn build<F>(builder: &Self::Builder, grammar: &Grammar, conflict_resolution: F) -> Result<NaiveLR1Table, ConstructionError>
    where
        F: FnMut(Conflict) -> Result<Action, Conflict>;
}

impl<'a, T> LR1TableBuilderStrategy<'a> for T
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

#[allow(clippy::doc_markdown)]
/// For some state q of an LRk automaton, the longest common preceding subpath is the longest
/// sequence of edges a_1, .., a_n all paths from start node s to q are of the form
/// b.., a_1, .., a_n.
pub trait LongestCommonPrecedingSubpath {
    fn longest_common_preceding_subpath<'a>(&self, grammar: &'a Grammar) -> Vec<&'a [Symbol]>;
}

impl<T: inner::ItemSets> LongestCommonPrecedingSubpath for T {
    fn longest_common_preceding_subpath<'a>(&self, grammar: &'a Grammar) -> Vec<&'a [Symbol]> {
        (0..self.state_count()).map(|state| {
            let max_item = self.items(state).iter().max_by_key(|&item| self.pos(item)).unwrap();
            &grammar.alt(self.production(max_item))[..self.pos(max_item)]
        }).collect()
    }
}