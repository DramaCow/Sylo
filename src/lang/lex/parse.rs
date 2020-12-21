use super::{LexAnalyzer, LexCommand};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Token<'a> {
    pub lexeme: &'a str,
    pub class: usize,
}

pub struct LexParse<'a> {
    lex: &'a LexAnalyzer,
    text:  &'a str,
    // === internals ===
    index: usize,
}

pub enum LexParseError<'a> {
    NoNextToken {
        byte_pos: usize,
        prefix: &'a str,
        text: &'a str,
    },
}

impl<'a> LexParse<'a> {
    pub(crate) fn new(lex: &'a LexAnalyzer, text: &'a str) -> Self {
        Self {
            lex,
            text,
            index: 0,
        }
    }
}

impl<'a> Iterator for LexParse<'a> {
    type Item = Result<Token<'a>, LexParseError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.text.len() {
            let mut state = self.lex.start();
            let mut index = self.index;
            
            let mut last_accept_state = self.lex.sink();
            let mut last_accept_index = 0_usize;

            // simulate dfa until hit the sink state or end of text
            for byte in self.text[self.index..].bytes() {            
                if state == self.lex.sink() {
                    break;
                }
                
                if self.lex.accept(state).is_some() {
                    last_accept_state = state;
                    last_accept_index = index;
                }
                
                state = self.lex.step(state, byte);
                index += 1;
            }

            if let Some(class) = self.lex.accept(state) {
                // currently on an accept state
                let i = self.index;
                self.index = index;

                match self.lex.commands.get(class).unwrap() {
                    LexCommand::Emit => return Some(Ok(Token { lexeme: &self.text[i..self.index], class })),
                    LexCommand::Skip => (),
                };
            } else if let Some(class) = self.lex.accept(last_accept_state) {
                // landed on an accept state in the past
                let i = self.index;
                self.index = last_accept_index;

                match self.lex.commands.get(class).unwrap() {
                    LexCommand::Emit => return Some(Ok(Token { lexeme: &self.text[i..self.index], class })), 
                    LexCommand::Skip => (),
                };
            } else {
                // failed to match anything
                let i = self.index;
                self.index = usize::MAX; // forces next iteration to return None

                return Some(Err(LexParseError::NoNextToken {
                    byte_pos: i,
                    prefix: &self.text[..i],
                    text: &self.text[i..],
                }));
            }
        };
        
        None
    }
}

impl std::fmt::Debug for LexParseError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LexParseError::NoNextToken { byte_pos, prefix, text } => {
                write!(f, "Failed to match token starting at byte {}: \"{}\"", byte_pos, text_summary(prefix, text, (6, 6)))
            },
        }
    }
}

// =================
// === INTERNALS ===
// =================

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

fn text_summary(prefix: &str, suffix: &str, (pn, sn): (usize, usize)) -> String {
    let (pcount, plen) = upto(prefix.chars().rev(), pn);
    let (scount, slen) = upto(suffix.chars(), sn);

    if pcount < pn && scount < sn {
        format!("{}{}", prefix, suffix)
    } else if pcount >= pn && scount < sn {
        format!("..{}{}", &prefix[prefix.len()-plen..], suffix)
    } else if pcount < pn && scount >= sn {
        format!("{}{}..", prefix, &suffix[..slen])
    } else {
        format!("..{}{}..", &prefix[prefix.len()-plen..], &suffix[..slen])
    }
}

fn upto<I: Iterator<Item=char>>(iter: I, n: usize) -> (usize, usize) {
    let mut count = 0_usize;
    let mut len = 0_usize;
    let mut iter = iter;

    for _ in 0..n {
        if let Some(chr) = iter.next() {
            count += 1;
            len += chr.len_utf8();
        } else {
            break;
        }
    }

    (count, len)
}