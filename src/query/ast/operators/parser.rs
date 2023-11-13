use super::{EqOperator, GtOperator, LtOperator};
use crate::query::ast::parser::{arguments, expression};
use crate::query::parser::{operator_pair, ws};
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::map;
use nom::sequence::separated_pair;
use nom::IResult;

pub fn gt_operator_expr(str: &str) -> IResult<&str, GtOperator> {
    map(operator_pair("$gt",arguments((expression, expression))), GtOperator::from)(str)
}

pub fn lt_operator_expr(str: &str) -> IResult<&str, LtOperator> {
    map(operator_pair("$lt",arguments((expression, expression))), LtOperator::from)(str)
}

pub fn eq_operator_expr(str: &str) -> IResult<&str, EqOperator> {
    map(operator_pair("$eq",arguments((expression, expression))), EqOperator::from)(str)
}