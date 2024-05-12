pub mod parser;
mod match_operator;

use derive_more::From;
use crate::Dynamic;
use crate::query::ast::expression::{Expression, ExprLiteral};
use crate::query::{Context, Eval, EvalError};

#[derive(From,Debug)]
pub struct GtOperator {
    arg1: Expression,
    arg2: Expression,
}


impl Eval for GtOperator{
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        let arg1 = self.arg1.eval_with_context(context)?;
        let arg2 = self.arg2.eval_with_context(context)?;

        Ok(Dynamic::Bool(arg1 > arg2))
    }
}


#[derive(From,Debug)]
pub struct LtOperator {
    arg1: Expression,
    arg2: Expression,
}

impl Eval for LtOperator{
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        let arg1 = self.arg1.eval_with_context(context)?;
        let arg2 = self.arg2.eval_with_context(context)?;

        Ok(Dynamic::Bool(arg1 < arg2))
    }
}


#[derive(From,Debug)]
pub struct EqOperator {
    arg1: Expression,
    arg2: Expression,
}

impl Eval for EqOperator{
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        let arg1 = self.arg1.eval_with_context(context)?;
        let arg2 = self.arg2.eval_with_context(context)?;

        Ok(Dynamic::Bool(arg1 == arg2))
    }
}
