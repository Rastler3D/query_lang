use smartstring::alias::String;
use crate::query::ast::{AndOperator, ArrayIndex, BetweenOperator, Field, FieldOperator, GteOperator, InnerField, InOperator, LeafValue, LteOperator, MemberAccess, NeOperator, NotOperator, Operator, OrOperator, Predicate, Value, Variable, VariablePath};
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

use std::iter::once;
use std::ops::{Deref, Range};
use hashlink::LinkedHashMap;
use crate::Number;
use crate::query::ast::{EqOperator, GtOperator, LtOperator};
use crate::query::ast::parser::{arguments, expression};

const HIGH_SURROGATES: Range<u16> = 0xd800..0xdc00;
pub fn ws<'a, O, E: ParseError<&'a str>>(
    wrapped: impl Parser<&'a str, O, E>,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E> {
    delimited(multispace0, wrapped, multispace0)
}

pub fn predicate(str: &str) -> IResult<&str, Predicate> {
    alt((
        map(operators, Predicate::Operators),
        map(leaf, Predicate::Leaf),
    ))(str)
}

pub fn leaf(str: &str) -> IResult<&str, LeafValue> {
    map(value, LeafValue::from)(str)
}

pub fn value(str: &str) -> IResult<&str, Value> {
    alt((
        map(number, Value::from),
        map(string, Value::from),
        map(boolean,Value::from),
        map(array, Value::from),
        map(object, Value::from),
        map(null, |_|Value::Null),
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
        map(gt_operator, Operator::from),
        map(gte_operator, Operator::from),
        map(lt_operator, Operator::from),
        map(lte_operator, Operator::from),
        map(between_operator, Operator::from),
        map(eq_operator, Operator::from),
        map(ne_operator, Operator::from),
        map(in_operator, Operator::from),
        map(not_operator, Operator::from),
        map(and_operator, Operator::from),
        map(or_operator, Operator::from),
    ))(str)
}

pub fn gt_operator(str: &str) -> IResult<&str, GtOperator> {
    map(
        operator_pair("$gt", cut(value)),
        GtOperator::from,
    )(str)
}

pub fn gte_operator(str: &str) -> IResult<&str, GteOperator> {
    map(
        operator_pair("$gte", cut(value)),
        GteOperator::from,
    )(str)
}

pub fn lt_operator(str: &str) -> IResult<&str, LtOperator> {
    map(
        operator_pair("$lt", cut(value)),
        LtOperator::from,
    )(str)
}

pub fn lte_operator(str: &str) -> IResult<&str, LteOperator> {
    map(
        operator_pair("$lte", cut(value)),
        LteOperator::from,
    )(str)
}

pub fn eq_operator(str: &str) -> IResult<&str, EqOperator> {
    map(
        operator_pair("$eq", cut(value)),
        EqOperator::from,
    )(str)
}
pub fn ne_operator(str: &str) -> IResult<&str, NeOperator> {
    map(
        operator_pair("$ne", cut(value)),
        NeOperator::from,
    )(str)
}

pub fn between_operator(str: &str) -> IResult<&str, BetweenOperator> {
    map(
        operator_pair("$between", cut(arguments((value, value)))),
        BetweenOperator::from,
    )(str)
}

pub fn in_operator(str: &str) -> IResult<&str, InOperator> {
    map(
        operator_pair("$in", cut(array)),
        InOperator::from,
    )(str)
}

pub fn not_operator(str: &str) -> IResult<&str, NotOperator> {
    map(
        operator_pair("$not", cut(predicate)),
        NotOperator::from,
    )(str)
}
pub fn and_operator(str: &str) -> IResult<&str, AndOperator> {
    map(
        operator_pair("$and", cut(array_of(predicate))),
        AndOperator::from,
    )(str)
}

pub fn or_operator(str: &str) -> IResult<&str, OrOperator> {
    map(
        operator_pair("$or", cut(array_of(predicate))),
        OrOperator::from,
    )(str)
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

pub fn object_of<'a, O>(
    element: impl Parser<&'a str, O, nom::error::Error<&'a str>>,
) -> impl FnMut(&'a str) -> IResult<&'a str, LinkedHashMap<String, O>, error::Error<&'a str>> {
    map(
        delimited(
            ws(character('{')),
            separated_list0(
                ws(character(',')),
                separated_pair(ws(string), character(':'), ws(element)),
            ),
            cut(ws(character('}'))),
        ),
        FromIterator::from_iter,
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
            move || VariablePath::BaseVariable(Variable::from(str)),
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
                member: member.into(),
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

pub fn array(str: &str) -> IResult<&str, Vec<Value>> {
    array_of(value)(str)
}

pub fn null(str: &str) -> IResult<&str, ()> {
    get_value((), ws(tag("null")))(str)
}

pub fn object(str: &str) -> IResult<&str, LinkedHashMap<String, Value>> {
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
        map(double, Number::from),
    ))(str)
}

pub fn boolean(str: &str) -> IResult<&str, bool> {
    alt((
        get_value(false, ws(tag("false"))),
        get_value(true, ws(tag("true")))
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
        |x| x.unwrap_or_default().into()
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

