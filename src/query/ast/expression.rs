use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use derive_more::From;
use hashlink::LinkedHashMap;
use smallvec::SmallVec;
use crate::{Dynamic, Number, Object};
use crate::query::ast::{MatchOperator, VariablePath};
use crate::query::ast::operators::{EqOperator, GtOperator, LtOperator};
use crate::query::{Context, Eval, EvalError};
use smartstring::alias::String;
#[derive(From,Debug)]
pub enum Expression {
    #[from]
    Literal(ExprLiteral),
    #[from(ExprOperator)]
    Operator(Box<ExprOperator>),
    #[from]
    Variable(ExprVariable),
    #[from]
    FieldPath(ExprFieldPath),
    #[from]
    Precomputed(Dynamic)
}

impl Eval for Expression{
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        match self {
            Expression::Literal(literal) => literal.eval_with_context(context),
            Expression::Operator(operator) => operator.eval_with_context(context),
            Expression::Variable(variable) => variable.eval_with_context(context),
            Expression::FieldPath(field) => field.eval_with_context(context),
            Expression::Precomputed(value) => Ok(value.clone()),
        }
    }
}

#[derive(From,Debug)]
pub enum ExprLiteral{
    Null(NullLiteral),
    Number(NumberLiteral),
    Bool(BoolLiteral),
    String(StringLiteral),
    Array(ArrayLiteral),
    Object(ObjectLiteral)
}

impl Eval for ExprLiteral{
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        match self {
            ExprLiteral::Null(null_literal) => null_literal.eval_with_context(context),
            ExprLiteral::Number(number_literal) => number_literal.eval_with_context(context),
            ExprLiteral::Bool(bool_literal) => bool_literal.eval_with_context(context),
            ExprLiteral::String(string_literal) => string_literal.eval_with_context(context),
            ExprLiteral::Array(array_literal) => array_literal.eval_with_context(context),
            ExprLiteral::Object(object_literal) => object_literal.eval_with_context(context)
        }
    }
}

#[derive(From,Debug)]
pub struct NullLiteral;

impl Eval for NullLiteral{
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        Ok(Dynamic::Null)
    }
}

#[derive(From,Debug)]
pub struct BoolLiteral{
    value: bool
}


impl Eval for BoolLiteral{
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        Ok(Dynamic::from(self.value))
    }
}

#[derive(From,Debug)]
pub struct NumberLiteral{
    value: Number
}

impl Eval for NumberLiteral{
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        Ok(Dynamic::from(self.value))
    }
}

#[derive(From,Debug)]
#[from(forward)]
pub struct StringLiteral{
    value: String
}

impl Eval for StringLiteral{
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        Ok(Dynamic::from(self.value.clone()))
    }
}

#[derive(From,Debug)]
pub struct ArrayLiteral{
    value: Vec<Expression>
}

impl Eval for ArrayLiteral{
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        let mut array = SmallVec::with_capacity(self.value.len());

        for expr in &self.value{
            array.push(expr.eval_with_context(context)?)
        }

        Ok(Dynamic::from(array))
    }
}

#[derive(From,Debug)]
pub struct ObjectLiteral{
    value: LinkedHashMap<String,Expression>
}

impl Eval for ObjectLiteral{
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        let mut obj = LinkedHashMap::with_capacity(self.value.len());

        for (key, expr) in &self.value{
            obj.insert(key.clone(),expr.eval_with_context(context)?);
        }
        Ok(Dynamic::from(obj))
    }
}

#[derive(From,Debug)]
pub enum ExprOperator {
    Match(MatchOperator),
    Gt(GtOperator),
    Lt(LtOperator),
    Eq(EqOperator),
}

impl Eval for ExprOperator{
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        match self{
            ExprOperator::Gt(gt) => gt.eval_with_context(context),
            ExprOperator::Lt(lt) => lt.eval_with_context(context),
            ExprOperator::Eq(eq) => eq.eval_with_context(context),
            ExprOperator::Match(r#match) => r#match.eval_with_context(context)
        }
    }
}

#[derive(From,Debug)]
pub struct ExprVariable {
    var_path: VariablePath,
}

impl Eval for ExprVariable{
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        self.var_path
            .resolve(&context.map)
            .ok_or(EvalError::UndefinedVariable)
    }
}

#[derive(From,Debug)]
pub struct ExprFieldPath {
    field_path: VariablePath,
}

impl Eval for ExprFieldPath{
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        context
            .get_variable("ROOT")
            .and_then(|x| self.field_path.resolve(&x))
            .ok_or(EvalError::UndefinedVariable)
    }
}
