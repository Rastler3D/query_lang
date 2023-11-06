use crate::query::ast::{
    ArrayIndex, BaseField, Field, FieldOperator, FieldPath, InnerField, LeafValue, MemberAccess,
    Operator, Predicate,
};
use nom::branch::alt;
use nom::bytes::complete::{escaped, escaped_transform, is_not, tag, take, take_while_m_n};
use nom::character::complete::{
    alpha1, anychar, char as character, hex_digit1, i64, multispace0, none_of, one_of, satisfy, u64,
};
use nom::character::is_hex_digit;
use nom::combinator::{all_consuming, cond, consumed, cut, eof, flat_map, into, map, map_opt, map_parser, map_res, not, opt, recognize, rest, success, value, verify};
use nom::error::ParseError;
use nom::multi::{count, fold_many0, fold_many1, many0, many1, many_till, separated_list1};
use nom::number::complete::double;
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated};
use nom::Err::Error;
use nom::{error, IResult, Parser};
use serde_json::{Number, Value};
use std::borrow::Cow;
use std::iter::once;
use std::ops::{Deref, Range};

const HIGH_SURROGATES: Range<u16> = 0xd800..0xdc00;
pub fn ws<'a, O, E: ParseError<&'a str>>(
    wrapped: impl Parser<&'a str, O, E>,
) -> impl Parser<&'a str, O, E> {
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
pub fn field_operator(str: &str) -> IResult<&str, FieldOperator> {
    map(
        separated_pair(ws(field), character(':'), ws(predicate)),
        FieldOperator::from,
    )(str)
}
pub fn field(str: &str) -> IResult<&str, Field> {
    map_parser(string, |string: String| {
        map(preceded(not(character('$')), field_path), Field::from)(string.deref())
            .map(|(_, field)| (String::new(), field))
            .map_err(|error| error.map_input(|_| str))
    })(str)
}

pub fn field_path(str: &str) -> IResult<&str, FieldPath> {
    flat_map(is_not(".[]"), |str: &str| {
        fold_many0(
            preceded(not(eof), cut(inner_field)),
            || FieldPath::BaseField(BaseField::from(str.to_string())),
            |mut base, field| FieldPath::InnerField {
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
pub fn number(str: &str) -> IResult<&str, Number> {
    alt((
        map(terminated(i64, not(one_of(".eE"))), Number::from),
        map_opt(double, Number::from_f64),
    ))(str)
}

pub fn string(str: &str) -> IResult<&str, String> {
    delimited(character('"'), unescape_string, cut(character('"')))(str)
}

pub fn unescape_string(str: &str) -> IResult<&str, String> {
    map(
        opt(escaped_transform(
            none_of(r#"\""#),
            '\\',
            alt((unescape_character_inner, unescape_codepoint_inner)),
        )),
        Option::unwrap_or_default
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
        value('\n', tag(r#"n"#)),
        value('\r', tag(r#"r"#)),
        value('\t', tag(r#"t"#)),
        value('\u{08}', tag(r#"b"#)),
        value('\u{0C}', tag(r#"f"#)),
        value('\\', tag(r#"\"#)),
        value('\"', tag(r#"""#)),
        value('/', tag(r#"/"#)),
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
