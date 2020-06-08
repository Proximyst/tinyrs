use serde::{Deserialize, Serialize};
use std::io::{BufRead as _, BufReader, Cursor, Read};

pub mod v1;

pub type DimensionSize = u8;

#[derive(Debug, Serialize, Deserialize)]
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
    CharArray(DimensionSize),
    ByteArray(DimensionSize),
    ShortArray(DimensionSize),
    IntArray(DimensionSize),
    LongArray(DimensionSize),
    BooleanArray(DimensionSize),
    FloatArray(DimensionSize),
    DoubleArray(DimensionSize),
    ClassArray(DimensionSize, String),
}

impl JavaType {
    pub(crate) fn parse(cursor: &mut Cursor<&str>) -> Result<JavaType, JavaTypeError> {
        let mut buf = [0u8];

        match cursor.read(&mut buf[..]) {
            Ok(0) | Err(_) => return Err(JavaTypeError::UnexpectedEof),
            _ => (),
        }

        match buf[0] {
            b'V' => return Ok(JavaType::Void),
            b'C' => return Ok(JavaType::Char),
            b'B' => return Ok(JavaType::Byte),
            b'S' => return Ok(JavaType::Short),
            b'I' => return Ok(JavaType::Int),
            b'J' => return Ok(JavaType::Long),
            b'Z' => return Ok(JavaType::Boolean),
            b'F' => return Ok(JavaType::Float),
            b'D' => return Ok(JavaType::Double),
            b'L' => (),
            b'[' => return Self::parse_array(cursor),
            _ => return Err(JavaTypeError::InvalidType),
        }

        let mut name = Vec::with_capacity(1);
        match cursor.read_until(b';', &mut name) {
            Ok(0) | Err(_) => return Err(JavaTypeError::UnexpectedEof),
            _ => (),
        }
        let name = String::from_utf8(name)?;

        Ok(JavaType::Class(name))
    }

    fn parse_array(cursor: &mut Cursor<&str>) -> Result<JavaType, JavaTypeError> {
        loop {
            let ty = Self::parse(cursor)?;

            return Ok(match ty {
                JavaType::Void => return Err(JavaTypeError::VoidArray), // Invalid array
                JavaType::Char => JavaType::CharArray(1),
                JavaType::Byte => JavaType::CharArray(1),
                JavaType::Short => JavaType::CharArray(1),
                JavaType::Int => JavaType::CharArray(1),
                JavaType::Long => JavaType::CharArray(1),
                JavaType::Boolean => JavaType::BooleanArray(1),
                JavaType::Float => JavaType::CharArray(1),
                JavaType::Double => JavaType::CharArray(1),
                JavaType::Class(class) => JavaType::ClassArray(1, class),

                JavaType::CharArray(dim) => JavaType::CharArray(dim + 1),
                JavaType::ByteArray(dim) => JavaType::ByteArray(dim + 1),
                JavaType::ShortArray(dim) => JavaType::ShortArray(dim + 1),
                JavaType::IntArray(dim) => JavaType::IntArray(dim + 1),
                JavaType::LongArray(dim) => JavaType::LongArray(dim + 1),
                JavaType::BooleanArray(dim) => JavaType::BooleanArray(dim + 1),
                JavaType::FloatArray(dim) => JavaType::FloatArray(dim + 1),
                JavaType::DoubleArray(dim) => JavaType::DoubleArray(dim + 1),
                JavaType::ClassArray(dim, class) => JavaType::ClassArray(dim + 1, class),
            });
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TinyVersion {
    V1(Vec<v1::Entry>),
}

#[derive(thiserror::Error, Debug)]
pub enum TinyError {
    #[error("error on reading input")]
    IoError(#[from] std::io::Error),

    #[error("invalid version")]
    InvalidVersion,

    #[error("error on reading v1: {0:?}")]
    V1Error(#[from] v1::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum JavaTypeError {
    #[error("error on reading input")]
    IoError(#[from] std::io::Error),

    #[error("void arrays do not exist")]
    VoidArray,

    #[error("invalid type")]
    InvalidType,

    #[error("unexpected EOF")]
    UnexpectedEof,

    #[error("non-utf8 string")]
    NonUtf8String(#[from] std::string::FromUtf8Error),
}

impl TinyVersion {
    pub fn parse<R: Read>(mut reader: BufReader<R>) -> Result<TinyVersion, TinyError> {
        let mut header = String::new();
        reader.read_line(&mut header)?;
        let header = header.split('\t').collect::<Vec<_>>();
        let version = match header.first() {
            Some(ver) => ver,
            None => return Err(TinyError::InvalidVersion),
        };

        match version {
            &"v1" => Ok(TinyVersion::V1(v1::parse(reader)?)),
            _ => Err(TinyError::InvalidVersion),
        }
    }
}
