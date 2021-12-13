
extern crate sylo;
extern crate regex_deriv;

use sylo::ast::*;
use regex_deriv as re;

fn main() {
    let mut def: ParserDef<_, ()> = ParserDef::new();
    def.tokens.push(Token { name: "DIGIT".to_string(), regex: re::RegEx::range32('0' as u32, '9' as u32) });
    def.rules.push(Rule {
        name: "Expr".to_string(),
        expr: Expr::Opt(Box::new(Expr::Token("DIGIT".to_string()))),
    });
    let (lexicon, grammar) = def.compile();
    println!("{:?}", grammar.productions().into_iter().collect::<Vec<_>>())
}