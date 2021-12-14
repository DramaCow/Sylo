use std::ops::Range;
use std::str::CharIndices;
use unicode_xid::UnicodeXID;

pub struct Scan<'a> {
    input: &'a str,
    chars: CharIndices<'a>,
    next_char: Option<(usize, char)>,
}

#[derive(Debug)]
pub struct ScanError {
    // TODO
}

#[derive(Debug, Clone)]
pub enum Token<'a> {
    Equals,
    Bar,
    Opt,
    Star,
    Plus,
    LParen,
    RParen,
    Colon,
    Semi,
    Ident(&'a str),
    Code(&'a str),
    Literal(&'a str),
}

impl<'a> Scan<'a> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        let mut scan = Self { input, chars: input.char_indices(), next_char: None };
        scan.advance();
        scan
    }

    fn advance(&mut self) -> Option<(usize, char)> {
        self.next_char = self.chars.next();
        self.next_char
    }

    #[allow(clippy::range_plus_one)]
    fn code(&mut self, i0: usize) -> Result<Token<'a>, ScanError> {
        // TODO: consider '{'s that appear in comments
        self.advance();
        let mut balance = 0; // number of unclosed '{'
        loop {
            match self.next_char {
                Some((i, '"')) => todo!(),
                Some((i, '\'')) => todo!(),
                Some((_, '{')) => { self.advance(); balance += 1; },
                Some((i, '}')) => {
                    self.advance();
                    if balance == 0 {
                        return Ok(Token::Code(&self.input[i0+1..i]))
                    };
                    balance -= 1;
                },
                Some(_) => { self.advance(); continue; }
                None => { self.advance(); return Err(ScanError {}); },
            }
        }
    }

    fn ident_tail(&mut self) -> usize {
        loop {
            match self.next_char {
                Some((_, c)) if is_identifier_continue(c) => { self.advance(); },
                Some((i, _)) => return i,
                None => return self.input.len(),
            }
        }
    }

    fn string_or_char_literal(&mut self, terminal: char) -> Option<usize> {
        let mut escaped = false;
        loop {
            let (i, c) = self.next_char?;
            match (c, escaped) {
                ('\\', _) => escaped = true,
                (c, false) if c == terminal => { self.advance(); return Some(i) },
                (c, true) if c == terminal => escaped = false,
                _ => (),
            }
            self.advance();
        }
    }
}

impl<'a> Iterator for Scan<'a> {
    type Item = Result<Token<'a>, ScanError>;

    #[allow(clippy::never_loop)]
    #[allow(clippy::range_plus_one)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            return match self.next_char {
                Some((i, '=')) => { self.advance(); Some(Ok(Token::Equals)) },
                Some((i, '|')) => { self.advance(); Some(Ok(Token::Bar)) },
                Some((i, '?')) => { self.advance(); Some(Ok(Token::Opt)) },
                Some((i, '*')) => { self.advance(); Some(Ok(Token::Star)) },
                Some((i, '+')) => { self.advance(); Some(Ok(Token::Plus)) },
                Some((i, '(')) => { self.advance(); Some(Ok(Token::LParen)) },
                Some((i, ')')) => { self.advance(); Some(Ok(Token::RParen)) },
                Some((i, ':')) => { self.advance(); Some(Ok(Token::Colon)) },
                Some((i, ';')) => { self.advance(); Some(Ok(Token::Semi)) },
                Some((i, '{')) => Some(self.code(i)),
                Some((i, c)) if is_identifier_start(c) => {
                    self.advance();
                    match c {
                        'r' => {
                            match self.next_char {
                                Some((_, '"')) => todo!(),
                                _ => Some(Ok(Token::Ident(&self.input[i..self.ident_tail()]))),
                            }
                         },
                        _ => Some(Ok(Token::Ident(&self.input[i..self.ident_tail()]))),
                    }
                }
                Some((i, '"')) => {
                    self.advance();
                    match self.string_or_char_literal('"') {
                        Some(end) => Some(Ok(Token::Literal(&self.input[i+1..end]))),
                        None => Some(Err(ScanError {})),
                    }
                }
                Some((_, c)) if c.is_whitespace() => { self.advance(); continue; }
                Some((_, _)) => Some(Err(ScanError {})),
                None => None,
            }
        }
    }
}

fn is_identifier_start(c: char) -> bool {
    UnicodeXID::is_xid_start(c) || c == '_'
}

fn is_identifier_continue(c: char) -> bool {
    UnicodeXID::is_xid_continue(c) || c == '_'
}