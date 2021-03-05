#[derive(Debug, Clone, Copy)]
pub enum Action {
    Invalid,
    Accept,
    Shift(usize),
    Reduce(usize),
}

trait ParsingTable {
    fn action(&self, state: usize, word: Option<usize>) -> Action;
    fn goto(&self, state: usize, var: usize) -> Option<usize>;
}