use super::{LexAnalyzer, Command};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Token<'a> {
    pub lexeme: &'a str,
    pub start_index: usize,
    pub end_index: usize,
    pub class:  usize,
}

pub struct Scan<'a> {
    lex:   &'a LexAnalyzer,
    input: &'a str,
    index: usize,
}

#[derive(Debug)]
pub struct ScanError {
    pos: usize,
}

impl<'a> Scan<'a> {
    pub(crate) fn new(lex: &'a LexAnalyzer, input: &'a str) -> Self {
        Self {
            lex,
            input,
            index: 0,
        }
    }
}

impl<'a> Iterator for Scan<'a> {
    type Item = Result<Token<'a>, ScanError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.input.len() {
            let mut state = 0;
            let mut index = self.index;
            
            let mut last_accept_state = self.lex.sink();
            let mut last_accept_index = 0_usize;

            // simulate dfa until hit the sink state or end of input
            for byte in self.input[self.index..].bytes() {            
                if state == self.lex.sink() {
                    break;
                }
                
                if self.lex.classes[state].is_some() {
                    last_accept_state = state;
                    last_accept_index = index;
                }
                
                state = self.lex.step(state, byte);
                index += 1;
            }

            // currently on an accept state
            if let Some(class) = self.lex.classes[state] {
                let i = self.index;
                self.index = index;

                match self.lex.commands[class] {
                    Command::Emit => return Some(Ok(Token { lexeme: &self.input[i..self.index], start_index: i, end_index: self.index, class })),
                    Command::Skip => (),
                };
            // landed on an accept state in the past
            } else if let Some(class) = self.lex.classes[last_accept_state] {
                let i = self.index;
                self.index = last_accept_index;

                match self.lex.commands[class] {
                    Command::Emit => return Some(Ok(Token { lexeme: &self.input[i..self.index], start_index: i, end_index: self.index, class })), 
                    Command::Skip => (),
                };
            // failed to match anything
            } else {
                let i = self.index;
                self.index = usize::MAX; // forces next iteration to return None

                return Some(Err(ScanError { pos: i }));
            }
        };
        
        None
    }
}

// =================
// === INTERNALS ===
// =================

impl LexAnalyzer {
    fn sink(&self) -> usize { 
        self.classes.len() - 1
    }

    fn step(&self, id: usize, symbol: u8) -> usize {
        self.next[256 * id + symbol as usize]
    }
}