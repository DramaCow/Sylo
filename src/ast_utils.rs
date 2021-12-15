use crate::ast;

pub type Grammar = ast::Grammar<String, String>;
pub type Rule = ast::Rule<String, String>;
pub type NamedExpr = ast::NamedExpr<String, String>;
pub type Expr = ast::Expr<String, String>;