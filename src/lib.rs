mod java;
pub mod v1;

pub use self::java::*;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{line_ending, not_line_ending},
    combinator::{map, not},
    error::{context, ParseError},
    multi::separated_nonempty_list,
    sequence::{preceded, terminated, tuple},
    IResult,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TinyVersion {
    V1(Vec<v1::Entry>, Vec<String>),
}

pub(crate) fn whitespace<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    take_while(char::is_whitespace)(input)
}

pub(crate) fn identifier<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        "identifier",
        terminated(not_line_ending, not(alt((whitespace, line_ending)))),
    )(input)
}

fn parse_v1<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TinyVersion, E> {
    context(
        "parse tiny v1",
        preceded(
            tag("v1"),
            map(
                tuple((
                    terminated(
                        separated_nonempty_list(whitespace, map(identifier, str::to_owned)),
                        line_ending,
                    ),
                    self::v1::parse_entries,
                )),
                |(names, entries)| TinyVersion::V1(entries, names),
            ),
        ),
    )(input)
}

pub fn parse_tiny<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TinyVersion, E> {
    context(
        "parse tiny",
        // TODO(Proximyst): Use `alt` when v2 is added
        parse_v1,
    )(input)
}
