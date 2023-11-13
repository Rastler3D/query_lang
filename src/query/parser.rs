use crate::query::ast::{
    ArrayIndex, Field, FieldOperator, InnerField, LeafValue, MemberAccess, Operator, Predicate,
    Variable, VariablePath,
};
use nom::branch::alt;
use nom::bytes::complete::{escaped, escaped_transform, is_not, tag, take};
use nom::character::complete::{
    char as character, hex_digit1, i64, multispace0, none_of, one_of, u64,
};

use nom::combinator::{
    all_consuming, cond, cut, eof, flat_map, map, map_opt, map_parser, map_res, not, opt,
    value as get_value,
};
use nom::error::ParseError;
use nom::multi::{fold_many0, separated_list0, separated_list1};
use nom::number::complete::double;
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};

use nom::{error, IResult, Parser};
use serde_json::{Number, Value};

use std::iter::once;
use std::ops::{Deref, Range};

const HIGH_SURROGATES: Range<u16> = 0xd800..0xdc00;
pub fn ws<'a, O, E: ParseError<&'a str>>(
    wrapped: impl Parser<&'a str, O, E>,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E> {
    delimited(multispace0, wrapped, multispace0)
}

pub fn predicate(str: &str) -> IResult<&str, Predicate> {
    alt((
        map(leaf, Predicate::Leaf),
        map(operators, Predicate::Operators),
    ))(str)
}

pub fn leaf(str: &str) -> IResult<&str, LeafValue> {
    alt((map(number, LeafValue::from), map(string, LeafValue::from)))(str)
}

pub fn value(str: &str) -> IResult<&str, Value> {
    alt((
        map(number, serde_json::Value::from),
        map(string, serde_json::Value::from),
        map(array, serde_json::Value::from),
        map(object, serde_json::Value::from),
        map(null, |_| serde_json::Value::Null),
    ))(str)
}

pub fn operators(str: &str) -> IResult<&str, Vec<Operator>> {
    delimited(
        ws(character('{')),
        separated_list1(ws(character(',')), operator),
        ws(character('}')),
    )(str)
}

pub fn operator(str: &str) -> IResult<&str, Operator> {
    alt((
        map(field_operator, Operator::from),
        map(field_operator, Operator::from),
    ))(str)
}

pub fn operator_pair<'a, O, E: ParseError<&'a str>>(
    name: &'a str,
    args: impl Parser<&'a str, O, E>,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E> {
    preceded(
        pair(
            ws(tuple((character('"'), tag(name), character('"')))),
            character(':'),
        ),
        ws(args),
    )
}

pub fn array_of<'a, O, E: ParseError<&'a str>>(
    element: impl Parser<&'a str, O, E>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<O>, E> {
    delimited(
        ws(character('[')),
        separated_list0(ws(character(',')), ws(element)),
        cut(ws(character(']'))),
    )
}

pub fn field_operator(str: &str) -> IResult<&str, FieldOperator> {
    map(
        separated_pair(ws(field), character(':'), ws(predicate)),
        FieldOperator::from,
    )(str)
}
pub fn field(str: &str) -> IResult<&str, Field> {
    map_parser(
        escaped_string,
        map(preceded(not(character('$')), field_path), Field::from),
    )(str)
}

pub fn field_path(str: &str) -> IResult<&str, VariablePath> {
    flat_map(is_not(".[]"), |str: &str| {
        fold_many0(
            preceded(not(eof), cut(inner_field)),
            || VariablePath::BaseVariable(Variable::from(str.to_string())),
            |base, field| VariablePath::InnerField {
                base: Box::new(base),
                field,
            },
        )
    })(str)
}
pub fn inner_field(str: &str) -> IResult<&str, InnerField, error::Error<&str>> {
    alt((
        map(preceded(character('.'), is_not(".[]")), |member: &str| {
            InnerField::MemberAccess(MemberAccess {
                member: member.to_string(),
            })
        }),
        map(
            delimited(character('['), ws(u64), character(']')),
            |index| {
                InnerField::ArrayIndex(ArrayIndex {
                    index: index as usize,
                })
            },
        ),
    ))(str)
}

pub fn array(str: &str) -> IResult<&str, Vec<serde_json::Value>> {
    array_of(value)(str)
}

pub fn null(str: &str) -> IResult<&str, ()> {
    get_value((), ws(tag("null")))(str)
}

pub fn object(str: &str) -> IResult<&str, serde_json::Map<String, Value>> {
    map(
        delimited(
            ws(character('{')),
            separated_list0(
                ws(character(',')),
                separated_pair(ws(string), character(':'), ws(value)),
            ),
            ws(character('}')),
        ),
        FromIterator::from_iter,
    )(str)
}

pub fn number(str: &str) -> IResult<&str, Number> {
    alt((
        map(terminated(i64, not(one_of(".eE"))), Number::from),
        map_opt(double, Number::from_f64),
    ))(str)
}

pub fn string(str: &str) -> IResult<&str, String> {
    map_parser(escaped_string, unescape_string)(str)
}

pub fn escaped_string(str: &str) -> IResult<&str, &str> {
    delimited(
        character('"'),
        map(
            opt(escaped(
                none_of(r#"\""#),
                '\\',
                alt((
                    one_of(r#""\/bfnrt"#),
                    terminated(
                        character('u'),
                        map_parser(take(4usize), all_consuming(hex_digit1)),
                    ),
                )),
            )),
            Option::unwrap_or_default,
        ),
        cut(character('"')),
    )(str)
}

pub fn unescape_string(str: &str) -> IResult<&str, String> {
    map(
        opt(escaped_transform(
            none_of(r#"\""#),
            '\\',
            alt((unescape_character_inner, unescape_codepoint_inner)),
        )),
        Option::unwrap_or_default,
    )(str)
}

fn unescape_codepoint_inner(str: &str) -> IResult<&str, char> {
    flat_map(unicode_codepoint_inner, |surrogate| {
        map_res(
            cond(
                HIGH_SURROGATES.contains(&surrogate),
                preceded(character('\\'), unicode_codepoint_inner),
            ),
            move |low_surrogate| {
                char::decode_utf16(once(surrogate).chain(low_surrogate))
                    .next()
                    .unwrap()
            },
        )
    })(str)
}
fn unescape_character_inner(str: &str) -> IResult<&str, char> {
    alt((
        get_value('\n', tag(r#"n"#)),
        get_value('\r', tag(r#"r"#)),
        get_value('\t', tag(r#"t"#)),
        get_value('\u{08}', tag(r#"b"#)),
        get_value('\u{0C}', tag(r#"f"#)),
        get_value('\\', tag(r#"\"#)),
        get_value('\"', tag(r#"""#)),
        get_value('/', tag(r#"/"#)),
    ))(str)
}

fn unicode_codepoint_inner(str: &str) -> IResult<&str, u16> {
    preceded(
        character('u'),
        map_res(map_parser(take(4usize), all_consuming(hex_digit1)), |hex| {
            u16::from_str_radix(hex, 16)
        }),
    )(str)
}

pub fn unescape_codepoint(str: &str) -> IResult<&str, char> {
    preceded(character('\\'), unescape_codepoint_inner)(str)
}

pub fn unicode_codepoint(str: &str) -> IResult<&str, u16> {
    preceded(character('\\'), unicode_codepoint_inner)(str)
}

pub fn unescape_character(str: &str) -> IResult<&str, char> {
    preceded(character('\\'), unescape_character_inner)(str)
}
