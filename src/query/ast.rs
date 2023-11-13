mod expression;
mod operators;
mod parser;

use serde_json::{Number, Value};
#[derive(Debug)]
pub struct Filter {
    pub predicate: Predicate,
}
#[derive(Debug)]
pub enum Predicate {
    Leaf(LeafValue),
    Operators(Vec<Operator>),
}
#[derive(Debug)]
pub struct LeafValue {
    pub value: serde_json::Value,
}

impl From<Number> for LeafValue {
    fn from(value: Number) -> Self {
        LeafValue {
            value: Value::Number(value),
        }
    }
}
impl From<String> for LeafValue {
    fn from(value: String) -> Self {
        LeafValue {
            value: Value::String(value),
        }
    }
}
#[derive(Debug)]
pub enum Operator {
    Value(FieldOperator),
}

impl From<FieldOperator> for Operator {
    fn from(value: FieldOperator) -> Self {
        Operator::Value(value)
    }
}
#[derive(Debug)]
pub struct FieldOperator {
    pub field: Field,
    pub predicate: Predicate,
}

impl From<(Field, Predicate)> for FieldOperator {
    fn from((field, predicate): (Field, Predicate)) -> Self {
        FieldOperator { field, predicate }
    }
}
#[derive(Debug)]
pub struct Field(VariablePath);
impl From<VariablePath> for Field {
    fn from(field_path: VariablePath) -> Self {
        Field(field_path)
    }
}
#[derive(Debug, Clone)]
pub enum VariablePath {
    BaseVariable(Variable),
    InnerField { base: Box<VariablePath>, field: InnerField },
}
#[derive(Debug, Clone)]
pub struct Variable {
    pub field: String,
}

impl From<String> for Variable {
    fn from(field: String) -> Self {
        Variable { field }
    }
}
#[derive(Debug, Clone)]
pub struct MemberAccess {
    pub member: String,
}
#[derive(Debug, Clone)]
pub struct ArrayIndex {
    pub index: usize,
}

#[derive(Debug, Clone)]
pub enum InnerField {
    MemberAccess(MemberAccess),
    ArrayIndex(ArrayIndex),
}
