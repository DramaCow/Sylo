mod compile;
pub use self::compile::LexDef;

mod parse;
use self::parse::Parse;

#[derive(Debug, Clone)]
pub enum Command {
    Emit,
    Skip,
}

pub struct LexAnalyzer {
    labels:   Vec<String>,
    table:    Vec<usize>,
    classes:  Vec<Option<usize>>,
    commands: Vec<Command>,
}

impl LexAnalyzer {
    #[must_use]
    pub fn parse<'a>(&'a self, text: &'a str) -> Parse<'a> {
        Parse::new(&self, text)
    }
}

impl LexAnalyzer {
    fn sink(&self) -> usize { 
        self.classes.len() - 1
    }

    #[allow(clippy::unused_self)]
    fn start(&self) -> usize {
        0
    }

    fn step(&self, id: usize, symbol: u8) -> usize {
        self.table[256 * id + symbol as usize]
    }

    fn accept(&self, id: usize) -> Option<usize> {
        self.classes[id]
    }

    // fn row(&self, i: usize) -> &[usize] {
    //     &self.adj_mat[256*i..256*(i+1)]
    // }
}

#[cfg(test)]
mod tests;