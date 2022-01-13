use std::collections::{HashMap, hash_map::Entry::{Occupied, Vacant}};
use regex_deriv::RegEx;
use regex_deriv_syntax as re;
use crate::repr::MetaRepr;

/// An abstract description of a language. This is what is expected to be
/// produced from a successful parse of a metasyntactic language description
/// (i.e. via the meta parser).
pub struct MetaAST {
    pub tokens: Vec<Token>,
    pub rules: Vec<Rule>,
}

#[derive(Clone)]
pub struct Token {
    pub name: String,
    pub regex: RegEx,
}

pub struct Rule {
    pub name: String,
    pub expr: Expr,
}

pub struct NamedExpr {
    pub name: Option<String>,
    pub expr: Expr,
}

pub enum Expr {
    Literal(String),
    Token(String),
    Rule(String),
    Seq(Vec<NamedExpr>, String),
    Alt(Vec<Expr>),
    Opt(Box<Expr>),
    Star(Box<Expr>),
    Plus(Box<Expr>),
}

impl Expr {
    #[must_use]
    pub fn tag(self, name: Option<String>) -> NamedExpr {
        NamedExpr { name, expr: self }
    }
}

impl MetaAST {
    #[must_use]
    pub fn repr(&self) -> MetaRepr {
        MetaRepr::new(self)
    }

    #[must_use]
    pub fn unparse(&self) -> String {
        self._unparse_imp(unparse_named_expr_full)
    }

    #[must_use]
    pub fn dump(&self) -> String {
        self._unparse_imp(unparse_named_expr_anon)
    }
    
    fn _unparse_imp<F>(&self, unparse_named_expr: F) -> String
    where
        F: Fn(&mut String, &[NamedExpr], &String) + Copy,
    {
        let mut fmt = String::new();
        for rule in &self.rules {
            fmt.push_str(rule.name.as_ref());
            fmt.push_str(" = ");
            unparse_expr(&mut fmt, &rule.expr, unparse_named_expr);
            fmt.push_str(";\n");
        }
        fmt
    }
}

// =================
// === INTERNALS ===
// =================

fn unparse_expr<F>(fmt: &mut String, parent: &Expr, unparse_named_expr: F)
where
    F: Fn(&mut String, &[NamedExpr], &String) + Copy,
{
    match parent {
        Expr::Literal(literal) => { fmt.push('"'); fmt.push_str(literal); fmt.push('"'); },
        Expr::Token(ident) => { fmt.push('$'); fmt.push_str(ident.as_ref()) },
        Expr::Rule(ident) => fmt.push_str(ident.as_ref()),
        Expr::Seq(named_exprs, code) => { fmt.push('('); unparse_named_expr(fmt, named_exprs, code); fmt.push(')'); }
        Expr::Alt(exprs) => {
            fmt.push('(');
            unparse_expr(fmt, exprs.first().unwrap(), unparse_named_expr);
            for expr in exprs.iter().skip(1) {
                fmt.push_str(" | ");
                unparse_expr(fmt, expr, unparse_named_expr);
            }
            fmt.push(')');
        },
        Expr::Opt(expr) => { unparse_expr(fmt, expr, unparse_named_expr); fmt.push('?'); },
        Expr::Star(expr) => { unparse_expr(fmt, expr, unparse_named_expr); fmt.push('*'); },
        Expr::Plus(expr) => { unparse_expr(fmt, expr, unparse_named_expr); fmt.push('+'); },
    }
}

fn unparse_named_expr_full(fmt: &mut String, named_exprs: &[NamedExpr], code: &String) {
    for named_expr in named_exprs {
        if let Some(ident) = named_expr.name.as_ref() {
            fmt.push_str(ident.as_ref());
            fmt.push(':');
        }
        unparse_expr(fmt, &named_expr.expr, unparse_named_expr_full);
        fmt.push(' ');
    }
    fmt.push('{');
    fmt.push_str(code.as_ref());
    fmt.push('}');
}

fn unparse_named_expr_anon(fmt: &mut String, named_exprs: &[NamedExpr], _: &String) {
    unparse_expr(fmt, &named_exprs.first().unwrap().expr, unparse_named_expr_anon);
    for named_expr in named_exprs.iter().skip(1) {
        fmt.push(' ');
        unparse_expr(fmt, &named_expr.expr, unparse_named_expr_anon);
    }
}