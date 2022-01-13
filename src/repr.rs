use crate::ast;
use regex_deriv::RegEx;
use std::collections::{HashMap, hash_map::Entry::{Occupied, Vacant}};
use lr_parsing_tools::grammar::{Symbol, Symbol::{Terminal as Word, Variable as Var}};
use std::fmt::Write;

/// An intermediate representation of a language that is more amenable to
/// compilation into a usable parser. Typically, this is not constructed
/// directly, but instead is constructed from a [`MetaAST`](ast::MetaAST).
/// This representation doesn't have inline terminals, excludes "EBNF"
/// convenience operations such as *, +, ? etc., and flattens nested rules.
pub struct MetaRepr {
    pub tokens: Vec<ast::Token>,
    pub rules: Vec<Rule>,
}

#[derive(Clone)]
pub struct Rule {
    pub name: String,
    pub productions: Vec<Production>,
}

#[derive(Clone)]
pub struct Production {
    pub symbols: Vec<Symbol>,
    pub action: Action,
}

#[derive(Clone)]
pub enum Action {
    Forward,
    Seq(String),
    None,
    Option,
    Vec,
}

impl MetaRepr {
    pub fn new(grammar: &ast::MetaAST) -> Self {
        MetaReprBuilder::new(grammar).build()
    }

    pub fn dump(&self) -> String {
        let mut fmt = String::new();
        
        // let padding = self.tokens.iter().map(|token| token.name.len()).max().unwrap();
        // for (i, token) in self.tokens.iter().enumerate() {
        //     writeln!(fmt, "{:width$} \\\\ id = {}", token.name, i, width = padding);
        // }

        // writeln!(fmt, "---");

        for rule in &self.rules {
            let padding = rule.name.len() + 1;
            write!(fmt, "{} =", rule.name);

            for (i, production) in rule.productions.iter().enumerate() {
                if i > 0 {
                    write!(fmt, "{:width$}|", "", width = padding);
                }

                if production.symbols.is_empty() {
                    writeln!(fmt, " \u{03b5}");
                } else {
                    for symbol in &production.symbols {
                        match symbol.clone() {
                            Word(word) => write!(fmt, " \"{}\"", &self.tokens[word].name),
                            Var(var) => write!(fmt, " {}", &self.rules[var].name),
                        };
                    }
                    writeln!(fmt);
                }
            }
        }

        return fmt;
    }
}

// =================
// === INTERNALS ===
// =================

struct MetaReprBuilder<'a> {
    grammar: &'a ast::MetaAST,
    literals: HashMap<String, usize>,
    tokens: HashMap<String, usize>,
    variables: HashMap<String, usize>,
    rules: Vec<Rule>,
}

enum Ret {
    Symbol(Symbol),
    Production(Production),
    Alt(Vec<Production>),
    Star(Symbol),
    Plus(Symbol),
}

impl<'a> MetaReprBuilder<'a> {
    fn new(grammar: &'a ast::MetaAST) -> Self {
        let mut literals = HashMap::new();
        grammar.rules.iter().fold(0, |acc, rule| collect_literals(&rule.expr, &mut literals, acc));

        // println!("{:?}", literals);

        let tokens = grammar.tokens.iter().enumerate()            
            .map(|(i, token)| (token.name.to_string(), i + literals.len())).collect();

        // println!("{:?}", tokens);
        
        let mut count = 0;
        let mut variables = HashMap::new();
        for rule in &grammar.rules {
            match variables.entry(rule.name.to_string()) {
                Occupied(_) => panic!("Rule with that name already exists."),
                Vacant(entry) => { entry.insert(count); count += 1; }
            }
        }

        // println!("{:?}", variables);

        let mut builder = MetaReprBuilder {
            grammar,
            literals,
            tokens,
            variables,
            rules: vec![Rule { name: String::new(), productions: Vec::new() }; grammar.rules.len()]
        };

        for (i, rule) in grammar.rules.iter().enumerate() {
            let name = rule.name.to_string();
            builder.rules[i] = match builder.visit(&rule.expr) {
                Ret::Symbol(symbol) => Rule { name, productions: vec![mk_fwd_production(symbol)] },
                Ret::Production(production) => Rule { name, productions: vec![production] },
                Ret::Alt(productions) => Rule { name, productions },
                Ret::Star(symbol) => mk_star_rule(name, i, symbol),
                Ret::Plus(symbol) => mk_plus_rule(name, i, symbol),
            }
        }

        builder
    }

    fn add_rule(&mut self, productions: Vec<Production>) -> usize {
        let index = self.rules.len();
        self.rules.push(Rule { name: format!("<{}>", index), productions });
        index
    }

    fn add_star_rule(&mut self, symbol: Symbol) -> usize {
        let index = self.rules.len();
        self.rules.push(mk_star_rule(format!("<{}>", index), index, symbol));
        index
    }

    fn add_plus_rule(&mut self, symbol: Symbol) -> usize {
        let index = self.rules.len();
        self.rules.push(mk_plus_rule(format!("<{}>", index), index, symbol));
        index
    }

    fn get_symbol(&mut self, expr: &ast::Expr) -> Symbol {
        match self.visit(expr) {
            Ret::Symbol(symbol) => symbol,
            Ret::Production(production) => Var(self.add_rule(vec![production])),
            Ret::Alt(productions) => Var(self.add_rule(productions)),
            Ret::Star(symbol) => Var(self.add_star_rule(symbol)),
            Ret::Plus(symbol) => Var(self.add_plus_rule(symbol)),
        }
    }

    fn visit(&mut self, parent: &ast::Expr) -> Ret {
        match parent {
            ast::Expr::Literal(literal) => {
                Ret::Symbol(Word(self.literals[literal]))
            },
            ast::Expr::Token(ident) => {
                Ret::Symbol(Word(self.tokens[ident]))
            },
            ast::Expr::Rule(ident) => {
                Ret::Symbol(Var(self.variables[ident]))
            },
            ast::Expr::Seq(named_exprs, code) => {
                let symbols = named_exprs.iter().map(|named_expr| self.get_symbol(&named_expr.expr)).collect();
                Ret::Production(Production { symbols, action: Action::Seq(code.to_string()) })
            },
            ast::Expr::Alt(exprs) => {
                let mut new_productions = Vec::new();
                for expr in exprs {
                    match self.visit(expr) {
                        Ret::Symbol(symbol) => new_productions.push(mk_fwd_production(symbol)),
                        Ret::Production(production) => new_productions.push(production),
                        Ret::Alt(productions) => new_productions.extend(productions),
                        Ret::Star(symbol) => new_productions.push(mk_fwd_production(Var(self.add_star_rule(symbol)))),
                        Ret::Plus(symbol) => new_productions.push(mk_fwd_production(Var(self.add_plus_rule(symbol)))),
                    }
                }
                Ret::Alt(new_productions)
            },
            ast::Expr::Opt(expr) => {
                // E --> A?
                // becomes
                // E --> A | eps
                Ret::Alt(vec![
                    Production { symbols: vec![self.get_symbol(expr)], action: Action::Option },
                    Production { symbols: Vec::new(), action: Action::Option },
                ])
            },
            ast::Expr::Star(expr) => {
                // E --> A*
                // becomes
                // E  --> E0
                // E0 --> E0 A | eps
                Ret::Star(self.get_symbol(expr))
            },
            ast::Expr::Plus(expr) => {
                // E --> A+
                // becomes
                // E  --> E0
                // E0 --> E0 A | A
                Ret::Plus(self.get_symbol(expr))
            },
        }
    }

    fn build(self) -> MetaRepr {
        let mut literal_tokens: Vec<_> = self.literals.into_iter().collect();
        literal_tokens.sort_by_key(|entry| entry.1);

        let tokens: Vec<ast::Token> = literal_tokens.iter().map(|literal| ast::Token { name: literal.0.to_string(), regex: regex_deriv_syntax::literal(&literal.0) })
            .chain(self.grammar.tokens.iter().cloned())
            .collect();

        MetaRepr { tokens, rules: self.rules }
    }
}



fn collect_literals(parent: &ast::Expr, literals: &mut HashMap<String, usize>, count: usize) -> usize {
    match parent {
        ast::Expr::Literal(literal)    => { literals.entry(literal.to_string()).or_insert(count); count + 1 },
        ast::Expr::Token(_)
        | ast::Expr::Rule(_)           => count,
        ast::Expr::Seq(named_exprs, _) => named_exprs.iter().fold(count, |acc, named_expr| collect_literals(&named_expr.expr, literals, acc)),
        ast::Expr::Alt(exprs)          => exprs.iter().fold(count, |acc, expr| collect_literals(expr, literals, acc)),
        ast::Expr::Opt(expr)
        | ast::Expr::Star(expr)
        | ast::Expr::Plus(expr)        => collect_literals(expr, literals, count),
    }
}

fn mk_fwd_production(symbol: Symbol) -> Production {
    Production { symbols: vec![symbol], action: Action::Forward }
}

fn mk_star_rule(name: String, index: usize, symbol: Symbol) -> Rule {
    Rule { name, productions: vec![
        Production { symbols: vec![Var(index), symbol], action: Action::Vec },
        Production { symbols: Vec::new(), action: Action::Vec }
    ]}
}

fn mk_plus_rule(name: String, index: usize, symbol: Symbol) -> Rule {
    Rule { name, productions: vec![
        Production { symbols: vec![Var(index), symbol], action: Action::Vec },
        Production { symbols: vec![symbol], action: Action::Vec }
    ]}
}