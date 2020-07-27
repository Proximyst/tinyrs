use crate::{identifier, parse_java_type, whitespace, JavaType};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    character::complete::{line_ending, not_line_ending},
    combinator::map,
    error::{context, ParseError},
    multi::{many0, many1, many1_count},
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
            preceded(char('#'), terminated(not_line_ending, line_ending)),
            |c: &'a str| Entry::Comment(c.into()),
        ),
    )(input)
}

fn class<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Entry, E> {
    context(
        "class",
        preceded(
            terminated(tag("CLASS"), many1_count(whitespace)),
            map(
                many1(map(
                    terminated(identifier, alt((whitespace, line_ending))),
                    str::to_owned,
                )),
                |names| Entry::Class { names },
            ),
        ),
    )(input)
}

fn field<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Entry, E> {
    context(
        "field",
        preceded(
            terminated(tag("FIELD"), many1_count(whitespace)),
            map(
                tuple((
                    terminated(identifier, whitespace),
                    terminated(parse_java_type, whitespace),
                    many1(map(
                        terminated(identifier, alt((whitespace, line_ending))),
                        str::to_owned,
                    )),
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
            terminated(tag("METHOD"), many1_count(whitespace)),
            map(
                tuple((
                    terminated(identifier, whitespace),
                    preceded(char('('), terminated(many0(parse_java_type), char(')'))),
                    parse_java_type,
                    many1(map(
                        terminated(identifier, alt((whitespace, line_ending))),
                        str::to_owned,
                    )),
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
    context("v1 entries", many0(alt((comment, class, field, method))))(input)
}
