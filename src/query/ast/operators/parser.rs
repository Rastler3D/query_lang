use super::{EqOperator, GtOperator, LtOperator};
use crate::query::ast::parser::{arguments, expression, named_arguments};
use crate::query::ast::MatchOperator;
use crate::query::parser::{operator_pair, predicate};
use nom::branch::alt;
use nom::combinator::{cut, map};
use nom::IResult;

pub fn gt_operator_expr(str: &str) -> IResult<&str, GtOperator> {
    map(
        operator_pair("$gt", cut(arguments((expression, expression)))),
        GtOperator::from,
    )(str)
}

pub fn lt_operator_expr(str: &str) -> IResult<&str, LtOperator> {
    map(
        operator_pair("$lt", cut(arguments((expression, expression)))),
        LtOperator::from,
    )(str)
}

pub fn eq_operator_expr(str: &str) -> IResult<&str, EqOperator> {
    map(
        operator_pair("$eq", cut(arguments((expression, expression)))),
        EqOperator::from,
    )(str)
}

pub fn match_operator_expr(str: &str) -> IResult<&str, MatchOperator> {
    operator_pair(
        "$match",
        cut(alt((
            map(
                named_arguments((
                    operator_pair("predicate", predicate),
                    operator_pair("object", expression),
                )),
                MatchOperator::from,
            ),
            map(predicate, MatchOperator::from),
        ))),
    )(str)
}
