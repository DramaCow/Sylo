use std::ops::Range;
use std::marker::PhantomData;
use super::{LexAnalyzer, Command};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Token<'a, I>
where
    I: AsRef<[u8]> + ?Sized
{
    pub span: Range<usize>,
    pub class: usize,
    _phantom: PhantomData<&'a I>,
}

pub struct Scan<'a, I>
where
    I: ?Sized
{
    lex:   &'a LexAnalyzer,
    input: &'a I,
    index: usize,
}

#[derive(Debug)]
pub struct ScanError {
    pos: usize,
}

impl<'a, I> Scan<'a, I>
where
    I: AsRef<[u8]> + ?Sized
{
    pub(crate) fn new(lex: &'a LexAnalyzer, input: &'a I) -> Self {
        Self {
            lex,
            input,
            index: 0,
        }
    }
}

impl<'a, I> Iterator for Scan<'a, I> 
where
    I: AsRef<[u8]> + ?Sized
{
    type Item = Result<Token<'a, I>, ScanError>;

    fn next(&mut self) -> Option<Self::Item> {       
        while self.index < self.input.as_ref().len() {
            let mut state = 0;
            let mut index = self.index;
            
            let mut last_accept_state = self.lex.sink();
            let mut last_accept_index = 0_usize;

            // simulate dfa until hit the sink state or end of input
            for byte in self.input.as_ref()[self.index..].iter().copied() {            
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
                    Command::Emit => return Some(Ok(Token { span: i..self.index, class, _phantom: PhantomData })),
                    Command::Skip => (),
                };
            // landed on an accept state in the past
            } else if let Some(class) = self.lex.classes[last_accept_state] {
                let i = self.index;
                self.index = last_accept_index;

                match self.lex.commands[class] {
                    Command::Emit => return Some(Ok(Token { span: i..self.index, class, _phantom: PhantomData })), 
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