use std::ops::Range;

pub fn scan<I: AsRef<[u8]> + ?Sized>(input: &I) -> Scan<'_> {
    Scan {
        input: input.as_ref(),
        index: 0,
    }
}

pub struct Scan<'a> {
    input: &'a [u8],
    index: usize,
}

#[derive(Debug)]
pub struct ScanError {
    pub pos: usize,
}

#[derive(Debug)]
pub enum TokenType {
    STRING,
    CHAR,
    RANGE,
    AND,
    OR,
    DIFF,
    OPT,
    STAR,
    PLUS,
    NOT,
    LPAREN,
    RPAREN,
    TT_ERROR,
    TT_SKIP,
}

#[derive(Debug)]
pub struct Token {
    pub ttype: TokenType,
    pub span: Range<usize>,
}

impl<'a> Iterator for Scan<'a> {
    type Item = Result<Token, ScanError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.begin()
    }
}

// *** LEXER TABLE START ***

struct Context {
    start_index: usize,
    last_accept_ttype: TokenType,
    last_accept_index: usize,
}

impl Scan<'_> {
    fn begin(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.index >= self.input.len() {
            return None;
        }

        let ctx = Context {
            start_index: self.index,
            last_accept_ttype: TokenType::TT_ERROR,
            last_accept_index: 0,
        };

        self.state0(ctx)
    }

    fn s0(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        if self.index >= self.input.len() { return self.sink(ctx); }
        let ch = self.input[self.index];
        ctx.last_accept_ttype = TokenType::TT_SKIP;
        ctx.last_accept_index = self.index;
        if ch == 0x28 { self.index += 1; return self.s1(ctx); }
        if ch == 0x2b { self.index += 1; return self.s2(ctx); }
        if ch == 0x2d { self.index += 1; return self.s3(ctx); }
        if ch == 0x26 { self.index += 1; return self.s4(ctx); }
        if ch == 0x2a { self.index += 1; return self.s5(ctx); }
        if ch == 0x22 { self.index += 1; return self.s6(ctx); }
        if ch == 0x7c { self.index += 1; return self.s10(ctx); }
        if ch == 0x27 { self.index += 1; return self.s11(ctx); }
        if ch == 0x2e { self.index += 1; return self.s15(ctx); }
        if ch == 0x3f { self.index += 1; return self.s17(ctx); }
        if ch == 0x29 { self.index += 1; return self.s18(ctx); }
        if ch == 0x21 { self.index += 1; return self.s19(ctx); }
        if ch == 0x09 ||
           ch == 0x0a ||
           ch == 0x0d ||
           ch == 0x20 { self.index += 1; return self.s20(ctx); }
        self.begin()
    }

    fn s1(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        Some(Ok(Token { ttype: TokenType::LPAREN, span: ctx.start_index..self.index }))
    }

    fn s2(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        Some(Ok(Token { ttype: TokenType::PLUS, span: ctx.start_index..self.index }))
    }

    fn s3(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        Some(Ok(Token { ttype: TokenType::DIFF, span: ctx.start_index..self.index }))
    }

    fn s4(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        Some(Ok(Token { ttype: TokenType::AND, span: ctx.start_index..self.index }))
    }

    fn s5(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        Some(Ok(Token { ttype: TokenType::STAR, span: ctx.start_index..self.index }))
    }

    fn s6(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        if self.index >= self.input.len() { return self.sink(ctx); }
        let ch = self.input[self.index];
        if ch == 0x09 ||
           ch == 0x0a ||
           ch == 0x0d ||
           ch == 0x20 ||
           ch == 0x21 ||
           (0x23 <= ch && ch <= 0x5b) ||
           (0x5d <= ch && ch <= 0x7e) ||
           (0xa0 <= ch && ch <= 0xff) { self.index += 1; return self.s7(ctx); }
        if ch == 0x5c { self.index += 1; return self.s9(ctx); }
        self.sink(ctx)
    }

    fn s7(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        if self.index >= self.input.len() { return self.sink(ctx); }
        let ch = self.input[self.index];
        if ch == 0x09 ||
           ch == 0x0a ||
           ch == 0x0d ||
           ch == 0x20 ||
           ch == 0x21 ||
           (0x23 <= ch && ch <= 0x5b) ||
           (0x5d <= ch && ch <= 0x7e) ||
           (0xa0 <= ch && ch <= 0xff) { self.index += 1; return self.s7(ctx); }
        if ch == 0x22 { self.index += 1; return self.s8(ctx); }
        if ch == 0x5c { self.index += 1; return self.s9(ctx); }
        self.sink(ctx)
    }

    fn s8(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        Some(Ok(Token { ttype: TokenType::STRING, span: ctx.start_index..self.index }))
    }

    fn s9(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        if self.index >= self.input.len() { return self.sink(ctx); }
        let ch = self.input[self.index];
        if ch == 0x22 ||
           ch == 0x5c { self.index += 1; return self.s7(ctx); }
        self.sink(ctx)
    }

    fn s10(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        Some(Ok(Token { ttype: TokenType::OR, span: ctx.start_index..self.index }))
    }

    fn s11(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        if self.index >= self.input.len() { return self.sink(ctx); }
        let ch = self.input[self.index];
        if ch == 0x09 ||
           ch == 0x0a ||
           ch == 0x0d ||
           ch == 0x20 ||
           ch == 0x21 ||
           (0x23 <= ch && ch <= 0x5b) ||
           (0x5d <= ch && ch <= 0x7e) ||
           (0xa0 <= ch && ch <= 0xff) { self.index += 1; return self.s12(ctx); }
        if ch == 0x5c { self.index += 1; return self.s14(ctx); }
        self.sink(ctx)
    }

    fn s12(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        if self.index >= self.input.len() { return self.sink(ctx); }
        let ch = self.input[self.index];
        if ch == 0x27 { self.index += 1; return self.s13(ctx); }
        self.sink(ctx)
    }

    fn s13(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        Some(Ok(Token { ttype: TokenType::CHAR, span: ctx.start_index..self.index }))
    }

    fn s14(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        if self.index >= self.input.len() { return self.sink(ctx); }
        let ch = self.input[self.index];
        if ch == 0x22 ||
           ch == 0x5c { self.index += 1; return self.s12(ctx); }
        self.sink(ctx)
    }

    fn s15(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        if self.index >= self.input.len() { return self.sink(ctx); }
        let ch = self.input[self.index];
        if ch == 0x2e { self.index += 1; return self.s16(ctx); }
        self.sink(ctx)
    }

    fn s16(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        Some(Ok(Token { ttype: TokenType::RANGE, span: ctx.start_index..self.index }))
    }

    fn s17(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        Some(Ok(Token { ttype: TokenType::OPT, span: ctx.start_index..self.index }))
    }

    fn s18(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        Some(Ok(Token { ttype: TokenType::RPAREN, span: ctx.start_index..self.index }))
    }

    fn s19(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        Some(Ok(Token { ttype: TokenType::NOT, span: ctx.start_index..self.index }))
    }

    fn s20(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        if self.index >= self.input.len() { return self.sink(ctx); }
        let ch = self.input[self.index];
        if ch == 0x09 ||
           ch == 0x0a ||
           ch == 0x0d ||
           ch == 0x20 { self.index += 1; return self.s20(ctx); }
        self.begin()
    }

    fn sink(&mut self, mut ctx: Context) -> Option<<Self as Iterator>::Item> {
        if let TokenType::TT_ERROR = ctx.last_accept_ttype {
            let pos = self.index;
            self.index = usize::MAX; // forces next iteration to return None
            Some(Err(ScanError { pos }))
        } else if let TokenType::TT_SKIP = ctx.last_accept_ttype {
            self.index = ctx.last_accept_index;
            self.begin()
        } else {
            self.index = ctx.last_accept_index;
            Some(Ok(Token { ttype: ctx.last_accept_ttype, span: ctx.start_index..ctx.last_accept_index }))
        }
    }
}

// ***  LEXER TABLE END  ***
