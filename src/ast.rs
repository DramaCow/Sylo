use std::collections::{HashSet, HashMap, hash_map::Entry::{Occupied, Vacant}};
use crate::re::{self, RegEx};
use crate::lr::grammar::{Grammar, GrammarBuilder, Symbol, Symbol::Terminal as Word, Symbol::Variable as Var};

#[derive(Default)]
pub struct ParserDef{
    pub tokens: Vec<Token>,
    pub rules: Vec<Rule>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Ident(pub String);

#[derive(Clone)]
pub enum ActionCode {
    Forward,
    Seq(String),
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
    Seq(Vec<NamedExpr>, String),
    Alt(Vec<Expr>),
    Opt(Box<Expr>),
    Star(Box<Expr>),
    Plus(Box<Expr>),
}

impl ParserDef {
    /// # Panics
    #[must_use]
    pub fn compile(&self) -> (Vec<RegEx>, Grammar) {
        let translator = Translator::new(self);
        
        let lexicon = translator.literals.into_iter().map(|literal| crate::re::literal(&literal))
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

struct Translator {
    literals: HashSet<String>,
    litmap: HashMap<String, Symbol>,
    tokenmap: HashMap<Ident, Symbol>,
    varmap: HashMap<Ident, Symbol>,
    rules: Vec<Vec<Production>>,
}

#[derive(Clone)]
struct Production {
    symbols: Vec<Symbol>,
    action: ActionCode,
}

enum Ret {
    Symbol(Symbol),
    Production(Production),
    Alt(Vec<Production>),
}

impl Translator {
    fn new(def: &ParserDef) -> Self {
        let literals: HashSet<String> = {
            let mut literals = HashSet::new();
            for rule in &def.rules {
                collect_literals(&rule.expr, &mut literals);
            }
            literals
        };

        let litmap: HashMap<String, Symbol> = literals.iter().enumerate()
            .map(|(i, literal)| (literal.clone(), Word(i))).collect();

        let tokenmap: HashMap<Ident, Symbol> = def.tokens.iter().enumerate()            
            .map(|(i, token)| (token.name.clone(), Word(i + litmap.len()))).collect();
        
        let varmap: HashMap<Ident, Symbol> = {
            let mut varmap = HashMap::new();
            let mut varcount = 0;
            for rule in &def.rules {
                match varmap.entry(rule.name.clone()) {
                    Occupied(_) => panic!("Rule with that name already exists."),
                    Vacant(entry) => { entry.insert(Var(varcount)); varcount += 1; }
                }
            }
            varmap
        };

        let mut translator = Translator { literals, litmap, tokenmap, varmap, rules: vec![Vec::new(); def.rules.len()] };

        for i in 0..def.rules.len() {
            translator.rules[i] = match translator.visit(&def.rules[i].expr) {
                Ret::Symbol(symbol) => vec![Production { symbols: vec![symbol], action: ActionCode::Forward }],
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

    fn get_symbol(&mut self, expr: &Expr) -> Symbol {
        match self.visit(expr) {
            Ret::Symbol(symbol) => symbol,
            Ret::Production(production) => Var(self.add_rule(vec![production])),
            Ret::Alt(productions) => Var(self.add_rule(productions)),
        }
    }

    fn visit(&mut self, parent: &Expr) -> Ret {
        match parent {
            Expr::Literal(literal) => {
                Ret::Symbol(self.litmap[literal])
            },
            Expr::Token(ident) => {
                Ret::Symbol(self.tokenmap[ident])
            },
            Expr::Rule(ident) => {
                Ret::Symbol(self.varmap[ident])
            },
            Expr::Seq(named_exprs, code) => {
                let symbols = named_exprs.iter().map(|named_expr| self.get_symbol(&named_expr.expr)).collect();
                Ret::Production(Production { symbols, action: ActionCode::Seq(code.clone()) })
            },
            Expr::Alt(exprs) => {
                let mut new_productions = Vec::new();
                for expr in exprs {
                    match self.visit(expr) {
                        Ret::Symbol(symbol) => new_productions.push(Production { symbols: vec![symbol], action: ActionCode::Forward }),
                        Ret::Production(production) => new_productions.push(production),
                        Ret::Alt(productions) => new_productions.extend(productions),
                    }
                }
                Ret::Alt(new_productions)
            },
            Expr::Opt(expr) => {
                Ret::Alt(vec![
                    Production { symbols: vec![self.get_symbol(expr)], action: ActionCode::Option },
                    Production { symbols: Vec::new(), action: ActionCode::None },
                ])
            },
            Expr::Star(expr) => {
                let index = self.add_rule(Vec::new());
                let symbol = self.get_symbol(expr);
                self.rules[index].push(Production { symbols: vec![Var(index), symbol], action: ActionCode::Vec });
                self.rules[index].push(Production { symbols: Vec::new(), action: ActionCode::None });
                Ret::Symbol(Var(index))
            },
            Expr::Plus(expr) => {
                let index = self.add_rule(Vec::new());
                let symbol = self.get_symbol(expr);
                self.rules[index].push(Production { symbols: vec![Var(index), symbol], action: ActionCode::Vec });
                self.rules[index].push(Production { symbols: vec![symbol], action: ActionCode::Forward });
                Ret::Symbol(Var(index))
            },
        }
    }
}

fn collect_literals(parent: &Expr, literals: &mut HashSet<String>) {
    match parent {
        Expr::Literal(literal)    => { literals.insert(literal.clone()); },
        Expr::Token(_)
        | Expr::Rule(_)           => (),
        Expr::Seq(named_exprs, _) => { for named_expr in named_exprs { collect_literals(&named_expr.expr, literals) } }
        Expr::Alt(exprs)          => { for expr in exprs { collect_literals(expr, literals) } },
        Expr::Opt(expr)
        | Expr::Star(expr)
        | Expr::Plus(expr)        => { collect_literals(expr, literals) },
    }
}