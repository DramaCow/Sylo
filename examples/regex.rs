
#[macro_use] extern crate sylo;

// use sylo::cst::{CSTNodeProxy, CSTVarNodeProxy, CSTWordNodeProxy};
use sylo::lang::re::{self, RegEx};
use sylo::lang::ParseError::{Lex as LexError, Syn as SynError};
use sylo::cst::{CST, CSTNode, CSTNodeId};
use std::time::Instant;

fn compile(cst: &CST, id: CSTNodeId) -> RegEx {
    const ALT         : usize = 1;
    const CONJ        : usize = 2;
    const SEQ         : usize = 3;
    const UNARY       : usize = 4;
    const PARENTHESES : usize = 5;
    const RANGE       : usize = 6;

    match id.to_node(cst) {
        CSTNode::Leaf(leaf) => {
            re::literal(leaf.token(cst).lexeme)
        },
        CSTNode::Branch(branch) => {
            let mut iter = branch.children(cst);
            let first = iter.next().unwrap();

            match branch.var {
                ALT => {
                    iter.fold(compile(cst, first), |acc, x| acc.or(&compile(cst, x)))
                },
                CONJ => {
                    iter.fold(compile(cst, first), |acc, x| acc.and(&compile(cst, x)))
                },
                SEQ => {
                    iter.fold(compile(cst, first), |acc, x| acc.then(&compile(cst, x)))
                },
                UNARY => {
                    let second = iter.next().unwrap();
                    match first.to_node(cst) {
                        CSTNode::Leaf(leaf) => {
                            if leaf.token(cst).class == 10 {
                                compile(cst, second).not()
                            } else {
                                panic!()
                            }
                        },
                        CSTNode::Branch(branch) => {
                            if branch.var == 5 {
                                
                                todo!()
                                // compile(cst, first)
                            } else {
                                panic!()
                            }
                        },
                    }
                },
                PARENTHESES => {
                    todo!()
                },
                RANGE => {
                    let _second = iter.next().unwrap(); // range operator ignored
                    let third = iter.next().unwrap();

                    match (first.to_node(cst), third.to_node(cst)) {
                        (CSTNode::Leaf(l1), CSTNode::Leaf(l2)) => {
                            todo!()
                        },
                        _ => panic!()
                    }
                },
                _ => panic!(),
            }
        },
    }
}

fn main() {
    let timer = Instant::now();

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
            // You can skip the first node if desired.
            [skip] Expr_ : AltOrConjOrSeqOrUnaryOrFactor_,

            Alt         : ConjOrSeqOrUnaryOrFactor_ AltPlus_,
            Conj        : SeqOrUnaryOrFactor_ ConjPlus_,
            Seq         : UnaryOrFactor_ SeqPlus_,
            Unary       : Factor opt
                        | Factor star
                        | Factor plus
                        | not Factor,
            Factor      : Parentheses
                        | string
                        | char
                        | Range,
            Parentheses : lparen AltOrConjOrSeqOrUnaryOrFactor_ rparen,
            Range       : char range char,

            // dummy variables
            [skip] AltPlus_                       : AltPlus_ or ConjOrSeqOrUnaryOrFactor_
                                                  | or ConjOrSeqOrUnaryOrFactor_,
            [skip] ConjPlus_                      : ConjPlus_ ConjOp_
                                                  | ConjOp_,
            [skip] ConjOp_                        : and SeqOrUnaryOrFactor_
                                                  | diff SeqOrUnaryOrFactor_,
            [skip] SeqPlus_                       : SeqPlus_ UnaryOrFactor_
                                                  | UnaryOrFactor_,
            [skip] AltOrConjOrSeqOrUnaryOrFactor_ : Alt | Conj | Seq | Unary | Factor,
            [skip] ConjOrSeqOrUnaryOrFactor_      : Conj | Seq | Unary | Factor,
            [skip] SeqOrUnaryOrFactor_            : Seq | Unary | Factor,
            [skip] UnaryOrFactor_                 : Unary | Factor,
            
        }
    };

    let parser = def.compile().unwrap();

    let cst = parser.cst("(('A'..'Z' | 'a'..'z' | '_') ('A'..'Z' | 'a'..'z' | '0'..'9' | '_')*) - '_'+").unwrap();
    std::fs::write("_graph.dot", cst.dot(&parser)).unwrap();
    // let regex = compile(&cst, cst.root());
    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());  
}