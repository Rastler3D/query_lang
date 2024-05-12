use crate::query::ast::expression::{ExprFieldPath, ExprLiteral, ExprOperator, ExprVariable, Expression, NullLiteral, NumberLiteral, StringLiteral, BoolLiteral, ArrayLiteral, ObjectLiteral};
use crate::query::ast::operators::parser::{eq_operator_expr, gt_operator_expr, lt_operator_expr, match_operator_expr};
use crate::query::parser::{array_of, escaped_string, field_path, number, object, object_of, string, boolean, ws, predicate};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{all_consuming, cut, map, map_parser, peek, verify};
use nom::sequence::{delimited, preceded};
use nom::IResult;
use crate::query::ast::Predicate;
use crate::query::Script;
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

pub fn script(str: &str) -> IResult<&str, Script>{
    map(all_consuming(ws(expression)), Script::from)(str)
}

pub fn parse_predicate(str: &str) -> IResult<&str, Predicate>{
    all_consuming(ws(predicate))(str)
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
    alt((
        map(null_literal, ExprLiteral::from),
        map(number_literal, ExprLiteral::from),
        map(bool_literal, ExprLiteral::from),
        map(string_literal, ExprLiteral::from),
        map(array_literal, ExprLiteral::from),
        map(object_literal, ExprLiteral::from),
    ))(str)
}

fn null_literal(str: &str) -> IResult<&str, NullLiteral>{
    map(ws(tag("null")), |_| NullLiteral)(str)
}

fn number_literal(str: &str) -> IResult<&str, NumberLiteral>{
    map(ws(number), NumberLiteral::from)(str)
}

fn string_literal(str: &str) -> IResult<&str, StringLiteral>{
    map(ws(string), StringLiteral::from)(str)
}

fn bool_literal(str: &str) -> IResult<&str, BoolLiteral>{
    map(ws(boolean), BoolLiteral::from)(str)
}

fn array_literal(str: &str) -> IResult<&str, ArrayLiteral>{
    map(ws(array_of(expression)), ArrayLiteral::from)(str)
}

fn object_literal(str: &str) -> IResult<&str, ObjectLiteral>{
    map(ws(object_of(expression)), ObjectLiteral::from)(str)
}

fn operator_expr(str: &str) -> IResult<&str, ExprOperator> {
    delimited(
        preceded(ws(char('{')), verify(peek(escaped_string), |str: &str| str.starts_with('$'))),
        cut(alt((
            map(gt_operator_expr, ExprOperator::from),
            map(lt_operator_expr, ExprOperator::from),
            map(eq_operator_expr, ExprOperator::from),
            map(match_operator_expr, ExprOperator::from)
        ))),
        ws(char('}')),
    )(str)
}

fn field_path_expr(str: &str) -> IResult<&str, ExprFieldPath> {
    map_parser(
        escaped_string,
        map(preceded(char('$'), cut(field_path)), ExprFieldPath::from),
    )(str)
}

fn variable_expr(str: &str) -> IResult<&str, ExprVariable> {
    map_parser(
        escaped_string,
        map(preceded(tag("$$"), cut(field_path)), ExprVariable::from),
    )(str)
}


