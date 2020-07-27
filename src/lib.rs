mod java;
pub mod v1;

pub use self::java::*;

use nom::{
    bytes::complete::{tag_no_case, take_while},
    character::complete::{ space1, line_ending },
    combinator::map,
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

pub(crate) fn identifier<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    const IDENTIFIER_CHARS: &'static str =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789$_/";

    context("identifier", take_while(|c| IDENTIFIER_CHARS.contains(c)))(input)
}

fn parse_v1<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TinyVersion, E> {
    preceded(
        terminated(tag_no_case("v1"), space1),
        map(
            tuple((
                terminated(
                    separated_nonempty_list(
                        space1,
                        map(identifier, str::to_owned),
                    ),
                    line_ending,
                ),
                self::v1::parse_entries,
            )),
            |(names, entries)| TinyVersion::V1(entries, names),
        ),
    )(input)
}

pub fn parse_tiny<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TinyVersion, E> {
    // TODO(Proximyst): Use `alt` when v2 is added
    parse_v1(input)
}
