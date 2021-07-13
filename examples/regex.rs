
#[macro_use] extern crate sylo;

use sylo::lang::re;
use sylo::parser::{Precedence, Associativity, strategy};
use sylo::codegen;
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
    let parser = def.build(strategy::LR1).unwrap();
    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());  

    std::fs::write("lexer.c.txt", codegen::c::lexer(String::new(), "RegEx_Lexer", &parser.lexer).unwrap()).unwrap();

    // let text = "('A'..'Z' | 'a'..'z' | '_') ('A'..'Z' | 'a'..'z' | '0'..'9' | '_')* - '_'+";
}