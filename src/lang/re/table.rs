#[derive(Clone, Copy)]
pub enum Command {
    Skip,
    Emit,
}

pub trait LexTable {
    const START_STATE: usize = 0;
    fn step(&self, state: usize, symbol: u8) -> usize;
    fn class(&self, state: usize) -> Option<usize>;
    fn sink(&self) -> usize;
    fn command(&self, class: usize) -> Command;
}