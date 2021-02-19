use super::{LexAnalyzer, Command};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Token<'a> {
    pub lexeme: &'a str,
    pub class:  usize,
}

pub struct Parse<'a> {
    lex:   &'a LexAnalyzer,
    text:  &'a str,
    index: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pos: usize,
}

impl<'a> Parse<'a> {
    pub(crate) fn new(lex: &'a LexAnalyzer, text: &'a str) -> Self {
        Self {
            lex,
            text,
            index: 0,
        }
    }
}

impl<'a> Iterator for Parse<'a> {
    type Item = Result<Token<'a>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.text.len() {
            let mut state = 0;
            let mut index = self.index;
            
            let mut last_accept_state = self.lex.sink();
            let mut last_accept_index = 0_usize;

            // simulate dfa until hit the sink state or end of text
            for byte in self.text[self.index..].bytes() {            
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
                    Command::Emit => return Some(Ok(Token { lexeme: &self.text[i..self.index], class })),
                    Command::Skip => (),
                };
            // landed on an accept state in the past
            } else if let Some(class) = self.lex.classes[last_accept_state] {
                let i = self.index;
                self.index = last_accept_index;

                match self.lex.commands[class] {
                    Command::Emit => return Some(Ok(Token { lexeme: &self.text[i..self.index], class })), 
                    Command::Skip => (),
                };
            // failed to match anything
            } else {
                let i = self.index;
                self.index = usize::MAX; // forces next iteration to return None

                return Some(Err(ParseError { pos: i }));
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

// fn text_summary(prefix: &str, suffix: &str, (pn, sn): (usize, usize)) -> String {
//     fn upto<I: Iterator<Item=char>>(iter: I, n: usize) -> (usize, usize) {
//         let mut count = 0_usize;
//         let mut len = 0_usize;
//         let mut iter = iter;
    
//         for _ in 0..n {
//             if let Some(chr) = iter.next() {
//                 count += 1;
//                 len += chr.len_utf8();
//             } else {
//                 break;
//             }
//         }
    
//         (count, len)
//     }

//     let (pcount, plen) = upto(prefix.chars().rev(), pn);
//     let (scount, slen) = upto(suffix.chars(), sn);

//     if pcount < pn && scount < sn {
//         format!("{}{}", prefix, suffix)
//     } else if pcount >= pn && scount < sn {
//         format!("..{}{}", &prefix[prefix.len()-plen..], suffix)
//     } else if pcount < pn && scount >= sn {
//         format!("{}{}..", prefix, &suffix[..slen])
//     } else {
//         format!("..{}{}..", &prefix[prefix.len()-plen..], &suffix[..slen])
//     }
// }