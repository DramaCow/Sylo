
#[macro_use] extern crate sylo;

use sylo::lang::re;
use std::time::Instant;

fn main() {
    let c = re::non_compatibility_char()
    .diff(&re::literal("\\"))
    .diff(&re::literal("\""))
    .or(&re::literal("\\\\"))
    .or(&re::literal("\\\""));
    
    let def = parser_def! {
        {
            [skip] _ws: re::any(" \n\t\r").plus(),
            string:     re::literal("\"").then(&c.plus()).then(&re::literal("\"")),
            char:       re::literal("'").then(&c).then(&re::literal("'")),
            range:      re::literal(".."),
            and:        re::literal("&"),
            or:         re::literal("|"),
            diff:       re::literal("-"),
            opt:        re::literal("?"),
            star:       re::literal("*"),
            plus:       re::literal("+"),
            not:        re::literal("!"),
            lparen:     re::literal("("),
            rparen:     re::literal(")"),
        },
        {
            Expr : OrClause | AndClause | Sequence | Unary | Factor,

            OrClause    : ConjOrSeqOrUnaryOrFactor_ AltPlus_,
            AndClause   : SeqOrUnaryOrFactor_ ConjPlus_,
            Sequence    : UnaryOrFactor_ SeqPlus_,
            Unary       : Factor opt
                        | Factor star
                        | Factor plus
                        | not Factor,
            [skip] Factor      : Parentheses
                        | string
                        | char
                        | Range,
            Parentheses : lparen Expr rparen,
            Range       : char range char,

            [skip] AltPlus_                  : AltPlus_ or ConjOrSeqOrUnaryOrFactor_ | or ConjOrSeqOrUnaryOrFactor_,
            [skip] ConjPlus_                 : ConjPlus_ ConjOp_ | ConjOp_,
            [skip] ConjOp_                   : and SeqOrUnaryOrFactor_ | diff SeqOrUnaryOrFactor_,
            [skip] SeqPlus_                  : SeqPlus_ UnaryOrFactor_ | UnaryOrFactor_,
            [skip] ConjOrSeqOrUnaryOrFactor_ : AndClause | Sequence | Unary | Factor,
            [skip] SeqOrUnaryOrFactor_       : Sequence | Unary | Factor,
            [skip] UnaryOrFactor_            : Unary | Factor,
            
        }
    };

    let timer = Instant::now();
    let parser = def.compile().unwrap();
    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());  

    let timer2 = Instant::now();
    let text = "(('A'..'Z' | 'a'..'z' | '_') ('A'..'Z' | 'a'..'z' | '0'..'9' | '_')*) - '_'+";
    let cst = parser.cst(text).unwrap();
    println!("CST built in {:?}.", timer2.elapsed());
    
    std::fs::write("_graph.dot", cst.dot(&parser, text)).unwrap();
    // let regex = compile(&cst, cst.root());
}