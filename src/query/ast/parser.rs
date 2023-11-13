use crate::query::ast::expression::{
    ExprFieldPath, ExprLiteral, ExprOperator, ExprVariable, Expression,
};
use crate::query::ast::operators::parser::{eq_operator_expr, gt_operator_expr, lt_operator_expr};
use crate::query::ast::operators::{EqOperator, GtOperator, LtOperator};
use crate::query::parser::{array_of, escaped_string, field_path, object, value, ws};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{cut, map, map_parser, map_res};
use nom::multi::separated_list1;
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;
use serde_json::Value;
use crate::query::utils::{separated_permutation, separated_tuple, SeparatedPermutation, SeparatedTuple};

pub fn arguments<'a, O>(args: impl SeparatedTuple<&'a str, O, nom::error::Error<&'a str>>) -> impl FnMut(&'a str) -> IResult<&'a str, O> {
    delimited(
        ws(char('[')),
        separated_tuple(ws(char(',')), args),
        cut(ws(char(']'))),
    )
}

pub fn named_arguments<'a, O>(args: impl SeparatedPermutation<&'a str, O, nom::error::Error<&'a str>>) -> impl FnMut(&'a str) -> IResult<&'a str, O> {
    delimited(
        ws(char('{')),
        separated_permutation(ws(char(',')), args),
        cut(ws(char('}'))),
    )
}

pub fn expression(str: &str) -> IResult<&str, Expression> {
    alt((
        map(variable_expr, Expression::from),
        map(field_path_expr, Expression::from),
        map(operator_expr, Expression::from),
        map(literal_expr, Expression::from),
    ))(str)
}

fn literal_expr(str: &str) -> IResult<&str, ExprLiteral> {
    map(value, ExprLiteral::from)(str)
}

fn operator_expr(str: &str) -> IResult<&str, ExprOperator> {
    delimited(
        ws(char('{')),
        alt((
            map(gt_operator_expr, ExprOperator::from),
            map(lt_operator_expr, ExprOperator::from),
            map(eq_operator_expr, ExprOperator::from),
        )),
        ws(char('}')),
    )(str)
}

fn field_path_expr(str: &str) -> IResult<&str, ExprFieldPath> {
    map_parser(
        escaped_string,
        map(preceded(char('$'), field_path), ExprFieldPath::from),
    )(str)
}

fn variable_expr(str: &str) -> IResult<&str, ExprVariable> {
    map_parser(
        escaped_string,
        map(preceded(tag("$$"), field_path), ExprVariable::from),
    )(str)
}


