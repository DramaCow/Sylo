#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::marker::PhantomData;
use std::ops::Range;
use crate::bindings;

pub fn scan<I: AsRef<[u8]> + ?Sized>(input: &I) -> Scan<'_, I> {
    Scan {
        scan: unsafe { bindings::RegEx_Lexer_new(input.as_ref().as_ptr(), input.as_ref().len() as std::os::raw::c_ulong) },
        marker: PhantomData,
    }
}

pub struct Scan<'a, I: ?Sized> {
    scan: bindings::RegEx_Lexer,
    marker: PhantomData<&'a I>,
}

pub struct ScanError {
    pub pos: usize,
}

pub enum TokenType {
    String,
    Char,
    Range,
    And,
    Or,
    Diff,
    Opt,
    Star,
    Plus,
    Not,
    Lparen,
    Rparen,
    TT_ERROR,
    TT_SKIP,
}

pub struct Token {
    pub ttype: TokenType,
    pub span: Range<usize>,
}

impl From<bindings::RegEx_Lexer_Token> for Token {
    fn from(token: bindings::RegEx_Lexer_Token) -> Self {
        let ttype = match token.type_ {
            bindings::RegEx_Lexer_TokenType_STRING   => TokenType::String,
            bindings::RegEx_Lexer_TokenType_CHAR     => TokenType::Char,
            bindings::RegEx_Lexer_TokenType_RANGE    => TokenType::Range,
            bindings::RegEx_Lexer_TokenType_AND      => TokenType::And,
            bindings::RegEx_Lexer_TokenType_OR       => TokenType::Or,
            bindings::RegEx_Lexer_TokenType_DIFF     => TokenType::Diff,
            bindings::RegEx_Lexer_TokenType_OPT      => TokenType::Opt,
            bindings::RegEx_Lexer_TokenType_STAR     => TokenType::Star,
            bindings::RegEx_Lexer_TokenType_PLUS     => TokenType::Plus,
            bindings::RegEx_Lexer_TokenType_NOT      => TokenType::Not,
            bindings::RegEx_Lexer_TokenType_LPAREN   => TokenType::Lparen,
            bindings::RegEx_Lexer_TokenType_RPAREN   => TokenType::Rparen,
            bindings::RegEx_Lexer_TokenType_TT_ERROR => TokenType::TT_ERROR,
            bindings::RegEx_Lexer_TokenType_TT_SKIP  => TokenType::TT_SKIP,
            _ => unreachable!(),
        };
        Token { ttype, span: token.span_start as usize..token.span_end as usize }
    }
}

impl<'a, I: AsRef<[u8]> + ?Sized> Iterator for Scan<'a, I> {
    type Item = Result<Token, ScanError>;

    fn next(&mut self) -> Option<Self::Item> {
        let ptr: *mut _ = &mut self.scan;
        let retval = unsafe { bindings::RegEx_Lexer_next(ptr) };

        match retval.tag {
            bindings::RegEx_Lexer_Item_OK => Some(Ok(Token::from(unsafe { retval.__bindgen_anon_1.token }))),
            bindings::RegEx_Lexer_Item_ERR => Some(Err(ScanError { pos: unsafe { retval.__bindgen_anon_1.error.pos as usize } })),
            bindings::RegEx_Lexer_Item_NONE => None,
            _ => unreachable!(),
        }
    }
}