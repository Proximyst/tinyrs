use crate::{identifier, parse_java_type, JavaType};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, line_ending, not_line_ending, space1},
    combinator::map,
    error::{context, ParseError},
    multi::{many0, separated_nonempty_list},
    sequence::{preceded, terminated, tuple},
    IResult,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Entry {
    Comment(String),

    Class {
        names: Vec<String>,
    },

    Field {
        names: Vec<String>,
        owner: String,
        class: JavaType,
    },

    Method {
        names: Vec<String>,
        owner: String,
        arguments: Vec<JavaType>,
        return_type: JavaType,
    },
}

fn comment<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Entry, E> {
    context(
        "comment",
        map(
            // This will not take the last comment if the file does not end with a new line.
            // If the last comment is vital, the caller will simply have to tack on a new line.
            preceded(char('#'), terminated(not_line_ending, line_ending)),
            |c: &'a str| Entry::Comment(c.into()),
        ),
    )(input)
}

fn class<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Entry, E> {
    context(
        "class",
        preceded(
            terminated(tag("CLASS"), space1),
            map(
                terminated(
                    separated_nonempty_list(space1, map(identifier, str::to_owned)),
                    line_ending,
                ),
                |names| Entry::Class { names },
            ),
        ),
    )(input)
}

fn field<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Entry, E> {
    context(
        "field",
        preceded(
            terminated(tag("FIELD"), space1),
            map(
                tuple((
                    terminated(identifier, space1),
                    terminated(parse_java_type, space1),
                    terminated(
                        separated_nonempty_list(space1, map(identifier, str::to_owned)),
                        line_ending,
                    ),
                )),
                |(owner, class, names)| Entry::Field {
                    names,
                    owner: owner.into(),
                    class,
                },
            ),
        ),
    )(input)
}

fn method<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Entry, E> {
    context(
        "method",
        preceded(
            terminated(tag("METHOD"), space1),
            map(
                tuple((
                    terminated(identifier, space1),
                    preceded(char('('), terminated(many0(parse_java_type), char(')'))),
                    terminated(parse_java_type, space1),
                    terminated(
                        separated_nonempty_list(space1, map(identifier, str::to_owned)),
                        line_ending,
                    ),
                )),
                |(owner, arguments, return_type, names)| Entry::Method {
                    names,
                    owner: owner.into(),
                    arguments,
                    return_type,
                },
            ),
        ),
    )(input)
}

pub fn parse_entries<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Vec<Entry>, E> {
    context("v1 entries", many0(alt((comment, class, method, field))))(input)
}
