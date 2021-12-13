use std::collections::{HashSet, HashMap, hash_map::Entry::{Occupied, Vacant}};
use crate::re::{self, RegEx};
use crate::lr::grammar::{Grammar, GrammarBuilder, Symbol, Symbol::Terminal as Word, Symbol::Variable as Var};

pub struct ParserDef<Ident, Code> {
    pub tokens: Vec<Token<Ident>>,
    pub rules: Vec<Rule<Ident, Code>>,
}

#[derive(Clone)]
pub enum Action<Code> {
    Forward,
    Seq(Code),
    None,
    Option,
    Vec,
}

pub struct Token<Ident> {
    pub name: Ident,
    pub regex: RegEx,
}

pub struct Rule<Ident, Code> {
    pub name: Ident,
    pub expr: Expr<Ident, Code>,
}

pub struct NamedExpr<Ident, Code> {
    pub name: Option<Ident>,
    pub expr: Expr<Ident, Code>,
}

pub enum Expr<Ident, Code> {
    Literal(String),
    Token(Ident),
    Rule(Ident),
    Seq(Vec<NamedExpr<Ident, Code>>, Code),
    Alt(Vec<Expr<Ident, Code>>),
    Opt(Box<Expr<Ident, Code>>),
    Star(Box<Expr<Ident, Code>>),
    Plus(Box<Expr<Ident, Code>>),
}

impl<Ident, Code> ParserDef<Ident, Code>
where
    Ident: Clone + Eq + std::hash::Hash,
    Code: Clone,
{
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { tokens: Vec::new(), rules: Vec::new() }
    }

    /// # Panics
    #[must_use]
    pub fn compile(&self) -> (Vec<RegEx>, Grammar) {
        let translator = Translator::new(self);
        
        let lexicon = translator.literals.keys().map(|literal| crate::re::literal(&literal))
            .chain(self.tokens.iter().map(|token| token.regex.clone()))
            .collect();

        let grammar = translator.rules.iter().fold(GrammarBuilder::new(), |acc, rule| {
            acc.rule(&rule.iter().map(|production| production.symbols.as_slice()).collect::<Vec<_>>())
        }).build().unwrap();

        (lexicon, grammar)
    }
}

// =================
// === INTERNALS ===
// =================

struct Translator<Ident, Code> {
    literals: HashMap<String, Symbol>,
    tokens: HashMap<Ident, Symbol>,
    variables: HashMap<Ident, Symbol>,
    rules: Vec<Vec<Production<Code>>>,
}

#[derive(Clone)]
struct Production<Code> {
    symbols: Vec<Symbol>,
    action: Action<Code>,
}

enum Ret<Code> {
    Symbol(Symbol),
    Production(Production<Code>),
    Alt(Vec<Production<Code>>),
}

impl<Ident, Code> Translator<Ident, Code>
where
    Ident: Clone + Eq + std::hash::Hash,
    Code: Clone,
{
    fn new(def: &ParserDef<Ident, Code>) -> Self {
        let literals = {
            let mut literals = HashMap::new();
            def.rules.iter().fold(0, |acc, rule| collect_literals(&rule.expr, &mut literals, acc));
            literals
        };

        let tokens: HashMap<Ident, Symbol> = def.tokens.iter().enumerate()            
            .map(|(i, token)| (token.name.clone(), Word(i + literals.len()))).collect();
        
        let variables: HashMap<Ident, Symbol> = {
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

    fn add_rule(&mut self, productions: Vec<Production<Code>>) -> usize {
        let index = self.rules.len();
        self.rules.push(productions);
        index
    }

    fn get_symbol(&mut self, expr: &Expr<Ident, Code>) -> Symbol {
        match self.visit(expr) {
            Ret::Symbol(symbol) => symbol,
            Ret::Production(production) => Var(self.add_rule(vec![production])),
            Ret::Alt(productions) => Var(self.add_rule(productions)),
        }
    }

    fn visit(&mut self, parent: &Expr<Ident, Code>) -> Ret<Code> {
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

fn collect_literals<Ident, Code>(parent: &Expr<Ident, Code>, literals: &mut HashMap<String, Symbol>, count: usize) -> usize {
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