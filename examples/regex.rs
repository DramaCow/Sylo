
#[macro_use] extern crate sylo;

use sylo::lang::{
    re,
    lr::Precedence,
    lr1::LR1ABuilder,
};
use std::time::Instant;

fn main() {
    let c = re::non_compatibility_char()
    .diff(&re::literal("\\"))
    .diff(&re::literal("\""))
    .or(&re::literal("\\\\"))
    .or(&re::literal("\\\""));
    
    let def = parser_def! {
        {
            /* 0*/ [skip] _ws: re::any(" \n\t\r").plus(),
            /* 1*/ string:     re::literal("\"").then(&c.plus()).then(&re::literal("\"")),
            /* 2*/ char:       re::literal("'").then(&c).then(&re::literal("'")),
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
            Expr : Expr or   Expr
                 | Expr and  Expr
                 | Expr diff Expr
                //  | Expr Expr
                 | Expr opt
                 | Expr star
                 | Expr plus
                 | not Expr
                 | lparen Expr rparen
                 | string
                 | char
                 | char range char,
        }
    };

    println!("{:?}", def.token_precedence);
    println!("{:?}", def.production_precedence);

    let timer = Instant::now();

    let lr1a = LR1ABuilder::new(&def.grammar).build();
    std::fs::write("_graph.dot", lr1a.dot(&def.grammar, &def.lexer_def.vocab.symbolic_names, &def.var_names, true)).unwrap();


    let parser = def.compile().unwrap();
    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());  

    let timer2 = Instant::now();
    let text = "(('A'..'Z' | 'a'..'z' | '_') ('A'..'Z' | 'a'..'z' | '0'..'9' | '_')*) - '_'+";
    let cst = parser.cst(text).unwrap();
    println!("CST built in {:?}.", timer2.elapsed());
    
    std::fs::write("_graph.dot", cst.dot(&parser, text)).unwrap();
    // let regex = compile(&cst, cst.root());
}