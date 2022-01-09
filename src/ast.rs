use std::collections::{HashMap, hash_map::Entry::{Occupied, Vacant}};
use regex_deriv::{RegEx, ByteSet};
use regex_deriv_syntax as re;
use lr_parsing_tools::grammar::{self as imp, Symbol::{Terminal as Word, Variable as Var}};

pub type Ident = String;
pub type Code = String;

pub struct Grammar {
    pub tokens: Vec<Token>,
    pub rules: Vec<Rule>,
}

#[derive(Clone)]
pub enum Action {
    Forward,
    Seq(Code),
    None,
    Option,
    Vec,
}

pub struct Token {
    pub name: Ident,
    pub regex: RegEx,
}

pub struct Rule {
    pub name: Ident,
    pub expr: Expr,
}

pub struct NamedExpr {
    pub name: Option<Ident>,
    pub expr: Expr,
}

pub enum Expr {
    Literal(String),
    Token(Ident),
    Rule(Ident),
    Seq(Vec<NamedExpr>, Code),
    Alt(Vec<Expr>),
    Opt(Box<Expr>),
    Star(Box<Expr>),
    Plus(Box<Expr>),
}

impl Expr {
    #[must_use]
    pub fn tag(self, name: Option<Ident>) -> NamedExpr {
        NamedExpr { name, expr: self }
    }
}

impl Grammar {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { tokens: Vec::new(), rules: Vec::new() }
    }

    /// # Panics
    #[must_use]
    pub fn compile(&self) -> (Vec<RegEx>, imp::Grammar) {
        let translator = Translator::new(self);
        
        let lexicon: Vec<RegEx> = translator.literals.keys().map(|literal| regex_literal(&literal))
            .chain(self.tokens.iter().map(|token| token.regex.clone()))
            .collect();

        let grammar = translator.rules.iter().fold(imp::GrammarBuilder::new(), |acc, rule| {
            acc.rule(&rule.iter().map(|production| production.symbols.as_slice()).collect::<Vec<_>>())
        }).build().unwrap();

        (lexicon, grammar)
    }

    #[must_use]
    pub fn unparse(&self) -> String {
        self._unparse_imp(unparse_named_expr_full)
    }

    #[must_use]
    pub fn unparse_anon(&self) -> String {
        self._unparse_imp(unparse_named_expr_anon)
    }
    
    fn _unparse_imp<F>(&self, unparse_named_expr: F) -> String
    where
        F: Fn(&mut String, &[NamedExpr], &Code) + Copy,
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

pub(crate) fn regex_literal(text: &str) -> RegEx {
    text.bytes().fold(RegEx::empty(), |acc, byte| {
        acc.then(&RegEx::set(ByteSet::point(byte)))
    })
}

struct Translator {
    literals: HashMap<String, imp::Symbol>,
    tokens: HashMap<Ident, imp::Symbol>,
    variables: HashMap<Ident, imp::Symbol>,
    rules: Vec<Vec<Production>>,
}

#[derive(Clone)]
struct Production {
    symbols: Vec<imp::Symbol>,
    action: Action,
}

enum Ret {
    Symbol(imp::Symbol),
    Production(Production),
    Alt(Vec<Production>),
}

impl Translator {
    fn new(def: &Grammar) -> Self {
        let literals = {
            let mut literals = HashMap::new();
            def.rules.iter().fold(0, |acc, rule| collect_literals(&rule.expr, &mut literals, acc));
            literals
        };

        let tokens: HashMap<Ident, imp::Symbol> = def.tokens.iter().enumerate()            
            .map(|(i, token)| (token.name.clone(), Word(i + literals.len()))).collect();
        
        let variables: HashMap<Ident, imp::Symbol> = {
            let mut variables = HashMap::new();
            let mut count = 0;
            for rule in &def.rules {
                match variables.entry(rule.name.clone()) {
                    Occupied(_) => panic!("Rule with that name already exists."),
                    Vacant(entry) => { entry.insert(Var(count)); count += 1; }
                }
            }
            variables
        };

        let mut translator = Translator { literals, tokens, variables, rules: vec![Vec::new(); def.rules.len()] };

        for i in 0..def.rules.len() {
            translator.rules[i] = match translator.visit(&def.rules[i].expr) {
                Ret::Symbol(symbol) => vec![Production { symbols: vec![symbol], action: Action::Forward }],
                Ret::Production(production) => vec![production],
                Ret::Alt(productions) => productions,
            }
        }

        translator
    }

    fn add_rule(&mut self, productions: Vec<Production>) -> usize {
        let index = self.rules.len();
        self.rules.push(productions);
        index
    }

    fn get_symbol(&mut self, expr: &Expr) -> imp::Symbol {
        match self.visit(expr) {
            Ret::Symbol(symbol) => symbol,
            Ret::Production(production) => Var(self.add_rule(vec![production])),
            Ret::Alt(productions) => Var(self.add_rule(productions)),
        }
    }

    fn visit(&mut self, parent: &Expr) -> Ret {
        match parent {
            Expr::Literal(literal) => {
                Ret::Symbol(self.literals[literal])
            },
            Expr::Token(ident) => {
                Ret::Symbol(self.tokens[ident])
            },
            Expr::Rule(ident) => {
                Ret::Symbol(self.variables[ident])
            },
            Expr::Seq(named_exprs, code) => {
                let symbols = named_exprs.iter().map(|named_expr| self.get_symbol(&named_expr.expr)).collect();
                Ret::Production(Production { symbols, action: Action::Seq(code.clone()) })
            },
            Expr::Alt(exprs) => {
                let mut new_productions = Vec::new();
                for expr in exprs {
                    match self.visit(expr) {
                        Ret::Symbol(symbol) => new_productions.push(Production { symbols: vec![symbol], action: Action::Forward }),
                        Ret::Production(production) => new_productions.push(production),
                        Ret::Alt(productions) => new_productions.extend(productions),
                    }
                }
                Ret::Alt(new_productions)
            },
            Expr::Opt(expr) => {
                Ret::Alt(vec![
                    Production { symbols: vec![self.get_symbol(expr)], action: Action::Option },
                    Production { symbols: Vec::new(), action: Action::None },
                ])
            },
            Expr::Star(expr) => {
                let index = self.add_rule(Vec::new());
                let symbol = self.get_symbol(expr);
                self.rules[index].push(Production { symbols: vec![Var(index), symbol], action: Action::Vec });
                self.rules[index].push(Production { symbols: Vec::new(), action: Action::None });
                Ret::Symbol(Var(index))
            },
            Expr::Plus(expr) => {
                let index = self.add_rule(Vec::new());
                let symbol = self.get_symbol(expr);
                self.rules[index].push(Production { symbols: vec![Var(index), symbol], action: Action::Vec });
                self.rules[index].push(Production { symbols: vec![symbol], action: Action::Forward });
                Ret::Symbol(Var(index))
            },
        }
    }
}

fn collect_literals(parent: &Expr, literals: &mut HashMap<String, imp::Symbol>, count: usize) -> usize {
    match parent {
        Expr::Literal(literal)    => { literals.entry(literal.clone()).or_insert(Word(count)); count + 1 },
        Expr::Token(_)
        | Expr::Rule(_)           => count,
        Expr::Seq(named_exprs, _) => named_exprs.iter().fold(count, |acc, named_expr| collect_literals(&named_expr.expr, literals, acc)),
        Expr::Alt(exprs)          => exprs.iter().fold(count, |acc, expr| collect_literals(expr, literals, acc)),
        Expr::Opt(expr)
        | Expr::Star(expr)
        | Expr::Plus(expr)        => collect_literals(expr, literals, count),
    }
}

fn unparse_expr<F>(fmt: &mut String, parent: &Expr, unparse_named_expr: F)
where
    F: Fn(&mut String, &[NamedExpr], &Code) + Copy,
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

fn unparse_named_expr_full(fmt: &mut String, named_exprs: &[NamedExpr], code: &Code) {
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

fn unparse_named_expr_anon(fmt: &mut String, named_exprs: &[NamedExpr], _: &Code) {
    unparse_expr(fmt, &named_exprs.first().unwrap().expr, unparse_named_expr_anon);
    for named_expr in named_exprs.iter().skip(1) {
        fmt.push(' ');
        unparse_expr(fmt, &named_expr.expr, unparse_named_expr_anon);
    }
}