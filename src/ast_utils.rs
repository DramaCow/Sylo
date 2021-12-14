use crate::ast;

pub type Grammar = ast::ParserDef<String, String>;
pub type Rule = ast::Rule<String, String>;
pub type NamedExpression = ast::NamedExpr<String, String>;
pub type Expression = ast::Expr<String, String>;