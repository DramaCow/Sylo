
#[macro_use] extern crate sylo;

use sylo::lang::re;
use sylo::lang::{Parser, lex::ScanError, lr::{ParseError, ParseTreeNode}};
use std::time::Instant;

fn compile_regex(parser: &Parser, text: &str) -> Result<re::RegEx, ParseError<ScanError>> {
    let mut regex = re::RegEx::none();
    let mut stack: Vec<char> = Vec::new();

    for res in parser.parse(text) {
        match res? {
            ParseTreeNode::Word(word) => {
                match word.class {
                    /*string*/ 1  => { let regex = re::literal(&text[word.span.start+1..word.span.end-1]); },
                    /*char*/   2  => {},
                    /*range*/  3  => {},
                    /*and*/    4  => {},
                    /*or*/     5  => {},
                    /*diff*/   6  => {},
                    /*opt*/    7  => {},
                    /*star*/   8  => {},
                    /*plus*/   9  => {},
                    /*not*/    10 => {},
                    /*lparen*/ 11 => {},
                    /*rparen*/ 12 => {},
                    _ => panic!(),
                }
            },
            ParseTreeNode::Var { var, child_count } => {
                match var {
                    /*Expr*/     0 => {},
                    /*Clause*/   1 => {},
                    /*Sequence*/ 2 => {},
                    /*Term*/     3 => {},
                    /*Factor*/   4 => {},
                    _ => panic!(),
                }
            },
        }
    }
    
    Ok(regex)
}

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
            Expr     : Expr or Clause
                     | Clause,
            Clause   : Clause and Sequence
                     | Clause diff Sequence
                     | Sequence,
            Sequence : Sequence Term
                     | Term,
            Term     : Factor opt
                     | Factor star
                     | Factor plus
                     | not Factor
                     | Factor,
            Factor   : lparen Expr rparen
                     | string
                     | char
                     | char range char,
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