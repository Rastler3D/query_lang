pub mod parser;

use derive_more::From;
use crate::query::ast::expression::Expression;
use serde_json::Value;

#[derive(From)]
pub struct GtOperator {
    arg1: Expression,
    arg2: Expression,
}


#[derive(From)]
pub struct LtOperator {
    arg1: Expression,
    arg2: Expression,
}


#[derive(From)]
pub struct EqOperator {
    arg1: Expression,
    arg2: Expression,
}
