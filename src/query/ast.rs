pub mod expression;
pub mod operators;
pub mod parser;

use std::fmt::{Display, Formatter, Pointer, Write};
use std::ops::Deref;
use derive_more::From;
use hashlink::LinkedHashMap;
use smallvec::SmallVec;
use crate::{Dynamic, Number, Object};
use crate::query::ast::expression::Expression;
use crate::query::{Context, Eval, EvalError};
use smartstring::alias::String;

#[derive(Debug)]
pub struct MatchOperator {
    pub predicate: Predicate,
    pub object: Option<Expression>
}

impl Eval for MatchOperator{
    fn eval_with_context(&self, context: &mut Context) -> Result<Dynamic, EvalError> {
        let current_object = if let Some(ref object) = self.object{
            object.eval_with_context(context)?
        } else { context.get_root() };
        let result = context.set_current_in_scope(current_object, |context| {
            self.predicate.test(context)
        })?;

        Ok(Dynamic::from(result))
    }
}

impl From<Predicate> for MatchOperator{
    fn from(predicate: Predicate) -> Self {
        MatchOperator{
            predicate,
            object: None
        }
    }
}

impl From<(Predicate,Expression)> for MatchOperator{
    fn from((predicate,expression): (Predicate, Expression)) -> Self {
        MatchOperator{
            predicate,
            object: Some(expression)
        }
    }
}

trait TestPredicate{
    fn test(&self, context: &mut Context) -> Result<bool, EvalError>;
}
#[derive(Debug, From, PartialEq, Clone)]
pub enum Predicate {
    Leaf(LeafValue),
    Operators(Vec<Operator>),
}

impl TestPredicate for Predicate{
    fn test(&self, context: &mut Context) -> Result<bool, EvalError> {
        match self {
            Predicate::Leaf(leaf) => leaf.test(context),
            Predicate::Operators(operators) => {
                for operator in operators{
                    if !operator.test(context)? { return Ok(false) }
                }

                Ok(true)
            }
        }
    }
}

#[derive(Debug,From, PartialEq, Clone)]
#[from(forward)]
pub enum Value{
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(LinkedHashMap<String,Value>)
}

impl Display for Value{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => {f.write_str("null")}
            Value::Bool(bool) => {
                bool.fmt(f)
            }
            Value::Number(num) => {
                num.fmt(f)
            }
            Value::String(string) => {
                string.fmt(f)
            }
            Value::Array(array) => {
                array.fmt(f)
            }
            Value::Object(object) => {
                object.fmt(f)
            }
        }
    }
}

impl From<Value> for serde_json::Value {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => {
                serde_json::Value::Null
            }
            Value::Bool(bool) => {
                serde_json::Value::Bool(bool)
            }
            Value::Number(number) => {
                serde_json::Value::Number(serde_json::Number::from(number))
            }
            Value::String(string) => {
                serde_json::Value::String(string.to_string())
            }
            Value::Array(array) => {
                serde_json::Value::Array(array.into_iter().map(|x| serde_json::Value::from(x)).collect())
            }
            Value::Object(object) => {
                serde_json::Value::Object(serde_json::Map::from_iter(object.into_iter().map(|x| (x.0.to_string(), serde_json::Value::from(x.1)))))
            }
        }
    }
}

impl PartialEq<Dynamic> for Value{
    fn eq(&self, other: &Dynamic) -> bool {
        match (self, other) {
            (Value::Null, Dynamic::Null) => true,
            (Value::Number(number), Dynamic::Number(other_number)) => number.eq(other_number),
            (Value::Bool(bool), Dynamic::Bool(other_bool)) => bool.eq(other_bool),
            (Value::String(string), Dynamic::String(other_string)) => string.eq(other_string.deref()),
            (Value::Array(array), Dynamic::Array(other_array)) => {
                let other_array = other_array.read().unwrap();
                array.iter().eq(other_array.iter())
            },
            (Value::Object(object), Dynamic::Object(other_object)) => {
                match other_object {
                    Object::Map(map) => {
                        let Ok(map) = map.read() else { return false };
                        object.iter().eq_by(map.iter(), |x,y| x.0.eq(y.0) && x.1.eq(y.1))
                    }
                    Object::DynamicObject(dyn_object) => {
                        object.iter().eq_by(dyn_object.field_values(), |x,y| x.0.eq(y.0) && x.1.eq(&y.1))
                    }
                }
            }
            _ => false
        }
    }
}


#[derive(Debug,From, PartialEq, Clone)]
pub struct LeafValue(pub Value);

impl TestPredicate for LeafValue{
    fn test(&self, context: &mut Context) -> Result<bool, EvalError> {
        let current_object = context.get_current();
        Ok(self.0.eq(&current_object))
    }
}
#[derive(Debug,From, PartialEq, Clone)]
pub enum Operator {
    Field(FieldOperator),
    Eq(EqOperator),
    Gt(GtOperator),
    Gte(GteOperator),
    Lt(LtOperator),
    Lte(LteOperator),
    Between(BetweenOperator),
    Ne(NeOperator),
    In(InOperator),
    Not(NotOperator),
    Or(OrOperator),
    And(AndOperator),
    Exists(ExistsOperator),
    IsEmpty(IsEmptyOperator)

}

impl TestPredicate for Operator{
    fn test(&self, context: &mut Context) -> Result<bool, EvalError> {
        match self {
            Operator::Field(field_operator) => field_operator.test(context),
            _ => { unimplemented!() }
        }
    }
}

#[derive(From,Debug, PartialEq, Clone)]
pub struct GtOperator(pub Value);
#[derive(From,Debug, PartialEq, Clone)]
pub struct GteOperator(pub Value);
#[derive(From,Debug, PartialEq, Clone)]
pub struct LtOperator(pub Value);
#[derive(From,Debug, PartialEq, Clone)]
pub struct LteOperator(pub Value);
#[derive(From, Debug, PartialEq, Clone)]
pub struct EqOperator(pub Value);
#[derive(From,Debug, PartialEq, Clone)]
pub struct NeOperator(pub Value);
#[derive(From,Debug, PartialEq, Clone)]
pub struct BetweenOperator(pub Value, pub Value);
#[derive(From,Debug, PartialEq, Clone)]
pub struct InOperator(pub Vec<Value>);
#[derive(From,Debug, PartialEq, Clone)]
pub struct NotOperator(pub Predicate);
#[derive(From,Debug, PartialEq, Clone)]
pub struct OrOperator(pub Vec<Predicate>);
#[derive(From,Debug, PartialEq, Clone)]
pub struct AndOperator(pub Vec<Predicate>);
#[derive(From,Debug, PartialEq, Clone)]
pub struct ExistsOperator(pub bool);
#[derive(From,Debug, PartialEq, Clone)]
pub struct IsEmptyOperator(pub bool);

#[derive(Debug,From, PartialEq, Clone)]
pub struct FieldOperator {
    pub field: Field,
    pub predicate: Predicate,
}

impl TestPredicate for FieldOperator{
    fn test(&self, context: &mut Context) -> Result<bool, EvalError> {
        let current_object = context.get_current();
        let next_object = self
            .field
            .resolve(&current_object);
        context.set_current_in_scope(next_object, |context| {
            self.predicate.test(context)
        })
    }
}

#[derive(Debug,From, PartialEq, Clone)]
pub struct Field(VariablePath);

impl Field{
    fn resolve(&self, value: &Dynamic) -> Dynamic{
        self.0
            .resolve(value)
            .unwrap_or(Dynamic::Null)
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariablePath {
    BaseVariable(Variable),
    InnerField { base: Box<VariablePath>, field: InnerField },
}

impl Display for VariablePath{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VariablePath::BaseVariable(variable) => {
                Display::fmt(variable, f)
            }
            VariablePath::InnerField { base, field } => {
                Display::fmt(base, f)?;
                Display::fmt(field, f)
            }
        }
    }
}

impl VariablePath{
    pub fn resolve(&self, root: &Dynamic) -> Option<Dynamic>{
        match self {
            VariablePath::BaseVariable(var) => root.get_object_field(&var.field),
            VariablePath::InnerField { base, field } => {
                let base = base.resolve(root)?;
                match field {
                    InnerField::MemberAccess(member_access) => {
                        base.get_object_field(&member_access.member)
                    }
                    InnerField::ArrayIndex(array_index) => {
                        base.get_array_item(array_index.index)
                    }
                }
            }
        }
    }
}
#[derive(Debug, Clone,From, PartialEq)]
#[from(forward)]
pub struct Variable {
    pub field: String,
}

impl Display for Variable{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.field)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberAccess {
    pub member: String,
}

impl Display for MemberAccess{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(".{}", self.member))
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayIndex {
    pub index: usize,
}

impl Display for ArrayIndex{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}]", self.index))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InnerField {
    MemberAccess(MemberAccess),
    ArrayIndex(ArrayIndex),
}

impl Display for InnerField{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InnerField::MemberAccess(member_access) => member_access.fmt(f),
            InnerField::ArrayIndex(array_index) => array_index.fmt(f)

        }
    }
}
