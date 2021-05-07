#![allow(non_upper_case_globals)]

use super::{Symbol, Grammar, GrammarBuilder, First, Follow};

fn rr_expr_grammar() -> Grammar {
    const add: Symbol    = Symbol::Terminal(0);
    const sub: Symbol    = Symbol::Terminal(1);
    const mul: Symbol    = Symbol::Terminal(2);
    const div: Symbol    = Symbol::Terminal(3);
    const lparen: Symbol = Symbol::Terminal(4);
    const rparen: Symbol = Symbol::Terminal(5);
    const name: Symbol   = Symbol::Terminal(6);
    const num: Symbol    = Symbol::Terminal(7);
    // ---
    const Expr: Symbol   = Symbol::Variable(0);
    const Expr_: Symbol  = Symbol::Variable(1);
    const Term: Symbol   = Symbol::Variable(2);
    const Term_: Symbol  = Symbol::Variable(3);
    const Factor: Symbol = Symbol::Variable(4);
    

    GrammarBuilder::new()
        .rule(&[&[Term, Expr_]])
        .rule(&[&[add, Term, Expr_],
                &[sub, Term, Expr_],
                &[]])
        .rule(&[&[Factor, Term_]])
        .rule(&[&[mul, Factor, Term_],
                &[div, Factor, Term_],
                &[]])
        .rule(&[&[lparen, Expr, rparen],
                &[name],
                &[num]])
        .build().unwrap()
}

const eps: Option<usize>    = None;
const add: Option<usize>    = Some(0);
const sub: Option<usize>    = Some(1);
const mul: Option<usize>    = Some(2);
const div: Option<usize>    = Some(3);
const lparen: Option<usize> = Some(4);
const rparen: Option<usize> = Some(5);
const name: Option<usize>   = Some(6);
const num: Option<usize>    = Some(7);
const eof: Option<usize>    = None;
// ---
const Expr: usize   = 0;
const Expr_: usize  = 1;
const Term: usize   = 2;
const Term_: usize  = 3;
const Factor: usize = 4;

#[test]
fn first() {
    let grammar = rr_expr_grammar();
    let first = First::new(&grammar);
    assert_eq!(&first[Expr], &[lparen, name, num]);
    assert_eq!(&first[Expr_], &[eps, add, sub]);
    assert_eq!(&first[Term], &[lparen, name, num]);
    assert_eq!(&first[Term_], &[eps, mul, div]);
    assert_eq!(&first[Factor], &[lparen, name, num]);
}

#[test]
fn follow() {
    let grammar = rr_expr_grammar();
    let follow = Follow::new(&grammar, &First::new(&grammar));
    assert_eq!(&follow[Expr], &[eof, rparen]);
    assert_eq!(&follow[Expr_], &[eof, rparen]);
    assert_eq!(&follow[Term], &[eof, add, sub, rparen]);
    assert_eq!(&follow[Term_], &[eof, add, sub, rparen]);
    assert_eq!(&follow[Factor], &[eof, add, sub, mul, div, rparen]);
}