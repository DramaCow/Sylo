use std::str::CharIndices;
use unicode_xid::UnicodeXID;

type Spanned<T> = (usize, T, usize);

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
    Less,
    Greater,
    Ident(&'a str),
    Token(&'a str),
    Literal(&'a str),
    RegEx(&'a str),
    Code(&'a str),
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
    fn code(&mut self, i0: usize) -> Result<Spanned<Token<'a>>, ScanError> {
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
                        return Ok((i0+1, Token::Code(&self.input[i0+1..i]), i))
                    };
                    balance -= 1;
                },
                Some(_) => { self.advance(); continue; }
                None => { self.advance(); return Err(ScanError {}); },
            }
        }
    }

    fn regex(&mut self, i0: usize) -> Result<Spanned<Token<'a>>, ScanError> {
        let mut num_pounds: usize = 0;
        loop {
            match self.next_char {
                Some((_, '#')) => num_pounds += 1,
                Some((_, '"')) => { self.advance(); break; }
                _ => return Err(ScanError {}),
            }
            self.advance();
        }

        let mut state: usize = 0;
        loop {
            let (i, c) = self.next_char.ok_or_else(|| ScanError {})?;
            state = match (state, c) {
                (_, '"') => 1,
                (0, '#') => 0,
                (_, '#') => state + 1, 
                (_, _)   => 0,
            };
            self.advance();
            if state == num_pounds + 1 {
                let start = i0 + num_pounds + 1;
                let end = i - num_pounds;
                return Ok((start, Token::RegEx(&self.input[start..end]), end))
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
    type Item = Result<Spanned<Token<'a>>, ScanError>;

    #[allow(clippy::never_loop)]
    #[allow(clippy::range_plus_one)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            return match self.next_char {
                Some((i, '=')) => { self.advance(); Some(Ok((i, Token::Equals , i+1))) },
                Some((i, '|')) => { self.advance(); Some(Ok((i, Token::Bar    , i+1))) },
                Some((i, '?')) => { self.advance(); Some(Ok((i, Token::Opt    , i+1))) },
                Some((i, '*')) => { self.advance(); Some(Ok((i, Token::Star   , i+1))) },
                Some((i, '+')) => { self.advance(); Some(Ok((i, Token::Plus   , i+1))) },
                Some((i, '(')) => { self.advance(); Some(Ok((i, Token::LParen , i+1))) },
                Some((i, ')')) => { self.advance(); Some(Ok((i, Token::RParen , i+1))) },
                Some((i, ':')) => { self.advance(); Some(Ok((i, Token::Colon  , i+1))) },
                Some((i, ';')) => { self.advance(); Some(Ok((i, Token::Semi   , i+1))) },
                Some((i, '<')) => { self.advance(); Some(Ok((i, Token::Less   , i+1))) },
                Some((i, '>')) => { self.advance(); Some(Ok((i, Token::Greater, i+1))) },
                Some((i, '{')) => Some(self.code(i)),
                Some((i, c)) if is_identifier_start(c) => {
                    self.advance();
                    match c {
                        'r' => {
                            match self.next_char {
                                Some((_, '"')) | Some((_, '#')) => {
                                    Some(self.regex(i+1))
                                },
                                _ => {
                                    let end = self.ident_tail();
                                    Some(Ok((i, Token::Ident(&self.input[i..end]), end)))
                                },
                            }
                         },
                        _ => {
                            let end = self.ident_tail();
                            Some(Ok((i, Token::Ident(&self.input[i..end]), end)))
                        },
                    }
                }
                Some((i, '$')) => {
                    match self.advance() {
                        Some((_, c)) if is_identifier_start(c) => {
                            self.advance();
                            let end = self.ident_tail();
                            Some(Ok((i+1, Token::Token(&self.input[i+1..end]), end)))
                        },
                        _ => Some(Err(ScanError {})),
                    }
                }
                Some((i, '"')) => {
                    self.advance();
                    match self.string_or_char_literal('"') {
                        Some(end) => Some(Ok((i+1, Token::Literal(&self.input[i+1..end]), end))),
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