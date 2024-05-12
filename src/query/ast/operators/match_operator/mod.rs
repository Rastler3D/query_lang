use derive_more::From;
use crate::query::ast::expression::Expression;
use crate::query::ast::VariablePath;

#[derive(Debug)]
pub struct MatchOperator {
    pub predicate: Predicate,
    pub object: Option<Expression>
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
#[derive(Debug,From)]
pub enum Predicate {
    Leaf(LeafValue),
    Operators(Vec<Operator>),
}
#[derive(Debug,From)]
pub struct LeafValue {
    pub value: serde_json::Value,
}
#[derive(Debug,From)]
pub enum Operator {
    Value(FieldOperator),
}

#[derive(Debug,From)]
pub struct FieldOperator {
    pub field: Field,
    pub predicate: Predicate,
}

#[derive(Debug,From)]
pub struct Field(VariablePath);