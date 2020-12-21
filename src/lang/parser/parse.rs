use super::Parser;
use crate::lang::lex::{LexParse, Token, LexParseError};
use crate::lang::syn::{SynParse, SynParseError};

use std::collections::VecDeque;

struct Parse<'a, F: FnMut(Result<Token, LexParseError>) -> usize> {
    lex: LexParse<'a>,
    syn: SynParse<'a, std::iter::Map<LexParse<'a>, F>>,
}

fn to_usize(token: Token) -> usize {
    token.class
}

// enum ParseError<'a> {
//     Lex(LexParseError<'a>),
//     Syn(SynParseError),
// }

// impl<'a> Parse<'a> {
//     fn new(parser: &'a Parser, text: &'a str) -> Self {
//         Self {
//             lex: LexParse::new(&parser.lex, text),
//         }
//     }
// }