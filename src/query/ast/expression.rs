use derive_more::From;
use crate::query::ast::VariablePath;
use serde_json::Value;
use crate::query::ast::operators::{EqOperator, GtOperator, LtOperator};

#[derive(From)]
pub enum Expression {
    #[from]
    Literal(ExprLiteral),
    #[from(ExprOperator)]
    Operator(Box<ExprOperator>),
    #[from]
    Variable(ExprVariable),
    #[from]
    FieldPath(ExprFieldPath),
}

#[derive(From)]
pub struct ExprLiteral {
    literal: Value,
}

#[derive(From)]
pub enum ExprOperator {
    Gt(GtOperator),
    Lt(LtOperator),
    Eq(EqOperator),
}

#[derive(From)]
pub struct ExprVariable {
    var_path: VariablePath,
}

#[derive(From)]
pub struct ExprFieldPath {
    field_path: VariablePath,
}
