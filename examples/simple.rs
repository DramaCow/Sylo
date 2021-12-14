
extern crate sylo;
extern crate regex_deriv;

use sylo::ast::*;
use regex_deriv as re;

fn main() {
    let ident = re::RegEx::range32('a'.into(), 'z'.into()).or(&re::RegEx::range32('A'.into(), 'Z'.into())).plus();
    let code = re::literal("{").then(&re::literal("}"));
    
    let mut def: ParserDef<_, ()> = ParserDef::new();
    
    def.tokens.push(Token { name: "IDENT".to_string(), regex: ident });
    def.tokens.push(Token { name: "CODE".to_string(), regex: code });
    
    def.rules.push(Rule {
        name: "Grammar".to_string(),
        expr: Expr::Rule("Rule".into()).plus(),
    });
    def.rules.push(Rule {
        name: "Rule".to_string(),
        expr: Expr::Seq(vec![
                Expr::Literal("rule".into()).tag(None),
                Expr::Token("IDENT".into()).tag(None),
                Expr::Literal("=".into()).tag(None),
                Expr::Rule("Expr".into()).tag(None),
                Expr::Literal(";".into()).tag(None),
            ], ()),
    });
    def.rules.push(Rule {
        name: "Expr".to_string(),
        expr: Expr::Seq(vec![
                Expr::Rule("Production".into()).tag(None),
                Expr::Seq(vec![Expr::Literal("|".into()).tag(None), Expr::Rule("Production".into()).tag(None)], ()).tag(None)
            ], ())
    });
    def.rules.push(Rule {
        name: "Production".to_string(),
        expr: Expr::Seq(vec![Expr::Rule("ProductionTerm".into()).plus().tag(None), Expr::Token("CODE".into()).tag(None)], ()),
    });
    def.rules.push(Rule {
        name: "ProductionTerm".to_string(),
        expr: Expr::Alt(vec![
                Expr::Seq(vec![Expr::Token("IDENT".to_string()).tag(None), Expr::Literal(":".to_string()).tag(None), Expr::Rule("Term".into()).tag(None)], ()),
                Expr::Rule("Term".into()),
            ])
    });
    def.rules.push(Rule {
        name: "Term".to_string(),
        expr: Expr::Alt(vec![
                Expr::Seq(vec![Expr::Rule("Factor".into()).tag(None), Expr::Literal("?".into()).tag(None)], ()),
                Expr::Seq(vec![Expr::Rule("Factor".into()).tag(None), Expr::Literal("*".into()).tag(None)], ()),
                Expr::Seq(vec![Expr::Rule("Factor".into()).tag(None), Expr::Literal("+".into()).tag(None)], ()),
                Expr::Rule("Factor".into()),
            ])
    });
    def.rules.push(Rule {
        name: "Factor".to_string(),
        expr: Expr::Alt(vec![
                Expr::Seq(vec![
                    Expr::Literal("(".into()).tag(None),
                    Expr::Rule("Expr".into()).tag(None),
                    Expr::Literal("(".into()).tag(None),
                ], ()),
                Expr::Token("IDENT".into()),
            ])
    });



    
    let (lexicon, grammar) = def.compile();
    
    println!("{:?}", grammar.productions().into_iter().collect::<Vec<_>>())
}