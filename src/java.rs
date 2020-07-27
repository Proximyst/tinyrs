use nom::{
    branch::alt,
    bytes::complete::take_till1,
    character::complete::char,
    combinator::{map, value},
    error::{context, ParseError},
    multi::many1_count,
    sequence::{pair, preceded},
    IResult,
};
use serde::{Deserialize, Serialize};

pub type DimensionSize = usize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum JavaType {
    Void,
    Char,
    Byte,
    Short,
    Int,
    Long,
    Boolean,
    Float,
    Double,
    Class(String),
    Array(DimensionSize, Box<JavaType>),
}

fn dataless<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, JavaType, E> {
    context(
        "dataless",
        alt((
            value(JavaType::Void, char('V')),
            value(JavaType::Char, char('C')),
            value(JavaType::Byte, char('B')),
            value(JavaType::Short, char('S')),
            value(JavaType::Int, char('I')),
            value(JavaType::Long, char('J')),
            value(JavaType::Boolean, char('Z')),
            value(JavaType::Float, char('F')),
            value(JavaType::Double, char('D')),
        )),
    )(input)
}

fn class<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, JavaType, E> {
    context(
        "class",
        map(
            preceded(char('L'), take_till1(|c| c == ';')),
            |name: &str| JavaType::Class(name.into()),
        ),
    )(input)
}

fn array<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, JavaType, E> {
    context(
        "array",
        map(
            pair(many1_count(char('[')), parse_java_type),
            |(dim, java)| JavaType::Array(dim, Box::new(java)),
        ),
    )(input)
}

pub fn parse_java_type<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, JavaType, E> {
    alt((dataless, class, array))(input)
}
