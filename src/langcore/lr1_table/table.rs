#[derive(Debug, Clone, Copy)]
pub enum Action {
    Invalid,
    Accept,
    Shift(usize),  //< shift to a *state*
    Reduce(usize), //< reduce via a *production*
}

#[derive(Debug, Clone, Copy)]
pub struct Reduction {
    pub var: usize,
    pub count: usize,
}

pub trait LR1Table {
    const START_STATE: usize = 0;
    fn action(&self, state: usize, word: Option<usize>) -> Action;
    fn goto(&self, state: usize, var: usize) -> Option<usize>;
    fn reduction(&self, alt: usize) -> Reduction;
}

#[derive(Debug)]
pub struct NaiveLR1Table {
    pub actions:    Vec<Action>,        /// lookup what action to perform given state and word
    pub gotos:      Vec<Option<usize>>, /// lookup what state should be transitioned to after reduction
    pub reductions: Vec<Reduction>,     // production --> rule and number of symbols
    pub word_count: usize,
    pub var_count:  usize,
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