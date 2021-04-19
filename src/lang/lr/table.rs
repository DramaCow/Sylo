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

pub trait ParsingTable {
    const START_STATE: usize = 0;
    fn action(&self, state: usize, word: Option<usize>) -> Action;
    fn goto(&self, state: usize, var: usize) -> Option<usize>;
    fn reduction(&self, alt: usize) -> Reduction;
}