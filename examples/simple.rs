
extern crate sylo;
extern crate regex_deriv;

use sylo::ast::*;
use regex_deriv as re;

fn main() {
    let mut def = ParserDef::default();
    def.tokens.push(Token { name: Ident("DIGIT".into()), regex: re::RegEx::range32('0' as u32, '9' as u32) });
    def.rules.push(Rule {
        name: Ident("Expr".into()),
        expr: Expr::Opt(Box::new(Expr::Token(Ident("DIGIT".into())))),
    });
    let (lexicon, grammar) = def.compile();
    println!("{:?}", grammar.productions().into_iter().collect::<Vec<_>>())
}