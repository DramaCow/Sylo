
#[macro_use] extern crate sylo;

use sylo::lang::{
    re,
    cfg::{nullability, First},
    lr::LR1ABuilder,
};
use sylo::syntax::{
    Precedence,
    Associativity,
};
use std::time::Instant;

fn main() {
    let c = re::non_compatibility_char()
    .diff(&re::literal("\\"))
    .diff(&re::literal("\""))
    .or(&re::literal("\\\\"))
    .or(&re::literal("\\\""));
    
    let mut def = parser! {
        {
            /* 0*/ [skip] _ws: re::any(" \n\t\r").plus(),
            /* 1*/ string:     re::literal("\"").then(&c.plus()).then(&re::literal("\"")),
            /* 2*/ CHAR:       re::literal("'").then(&c).then(&re::literal("'")),
            /* 3*/ range:      re::literal(".."),
            /* 4*/ and:        re::literal("&"),
            /* 5*/ or:         re::literal("|"),
            /* 6*/ diff:       re::literal("-"),
            /* 7*/ opt:        re::literal("?"),
            /* 8*/ star:       re::literal("*"),
            /* 9*/ plus:       re::literal("+"),
            /*10*/ not:        re::literal("!"),
            /*11*/ lparen:     re::literal("("),
            /*12*/ rparen:     re::literal(")"),
        },
        {
            %left or
            %left and diff
        },
        {
            /* 0*/ Expr : Expr or   Expr
            /* 1*/      | Expr and  Expr
            /* 2*/      | Expr diff Expr
            /* 3*/      | Expr Expr
            /* 4*/      | Expr opt
            /* 5*/      | Expr star
            /* 6*/      | Expr plus
            /* 7*/      | not Expr
            /* 8*/      | lparen Expr rparen
            /* 9*/      | string
            /*10*/      | CHAR
            /*11*/      | CHAR range CHAR,
        }
    };

    def.set_production_precedence(3, Precedence { level: 2, associativity: Associativity::Left }); // associativity doesn't matter for this production
    
    let timer = Instant::now();

    let lr1a = LR1ABuilder::new(&def.grammar).build();
    std::fs::write("_graph.dot", lr1a.dot(&def.grammar, &def.lexer_def.vocab(), &def.var_names, true).unwrap()).unwrap();

    let parser = def.build().unwrap();
    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());  
    println!("{}", sylo::syntax::compile::c_render::render_lexer(&parser.lexer, "MyLexer").unwrap());
    
    let timer2 = Instant::now();
    let text = "('A'..'Z' | 'a'..'z' | '_') ('A'..'Z' | 'a'..'z' | '0'..'9' | '_')* - '_'+";
    let cst = parser.cst(text).unwrap();
    println!("CST built in {:?}.", timer2.elapsed());
    
    std::fs::write("_graph.dot", cst.dot(&parser).unwrap()).unwrap();
    // let regex = compile(&cst, cst.root());
}