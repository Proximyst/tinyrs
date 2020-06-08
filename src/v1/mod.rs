use crate::JavaType;
use serde::{Deserialize, Serialize};
use std::io::{BufRead as _, BufReader, Cursor, Read};

#[derive(Debug, Serialize, Deserialize)]
pub enum Entry {
    Comment(String),

    Class {
        names: Vec<String>,
    },

    Field {
        names: Vec<String>,
        owner: String,

        /// Type name if class uses id 0 namespace.
        class: JavaType,
    },

    Method {
        names: Vec<String>,
        owner: String,

        /// Type name if class uses id 0 namespace.
        arguments: Vec<JavaType>,

        /// Type name if class uses id 0 namespace.
        return_type: JavaType,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error on reading input")]
    IoError(#[from] std::io::Error),

    #[error("hit unexpected EOF")]
    UnexpectedEof,

    #[error("hit unexpected end of line")]
    UnexpectedEol,

    #[error("non-utf8 string")]
    NonUtf8String(#[from] std::string::FromUtf8Error),

    #[error("non-utf8 str")]
    NonUtf8Str(#[from] std::str::Utf8Error),

    #[error("unknown entry type: {0}")]
    UnknownEntryType(String),

    #[error("java type parsing error: {0}")]
    JavaTypeParsing(#[from] crate::JavaTypeError),
}

pub(crate) fn parse<R: Read>(mut reader: BufReader<R>) -> Result<Vec<Entry>, Error> {
    // u16::max_value() is used because there are a lot of mappings in this
    // tool's intended usecase.
    let mut entries = Vec::with_capacity(u16::max_value() as usize);

    let mut buffer = String::with_capacity(256);
    loop {
        buffer.clear();

        if let 0 = reader.read_line(&mut buffer)? {
            // EOF reached.
            break;
        }

        let entry = parse_entry(&buffer)?;
        entries.push(entry);
    }

    Ok(entries)
}

fn parse_entry(line: &str) -> Result<Entry, Error> {
    if line.starts_with('#') {
        let after_octothorpe = &line[1..];
        let after_octothorpe = after_octothorpe.trim();
        let after_octothorpe = after_octothorpe.to_string();
        return Ok(Entry::Comment(after_octothorpe));
    }

    let mut cursor = Cursor::new(line);

    let mut entry_type = Vec::with_capacity(6);
    match cursor.read_until(b'\t', &mut entry_type) {
        Ok(0) | Err(_) => return Err(Error::UnexpectedEof),
        _ => (),
    }
    let entry_type = std::str::from_utf8(&entry_type)?.trim();

    match entry_type {
        "CLASS" => parse_class(cursor),
        "FIELD" => parse_field(cursor),
        "METHOD" => parse_method(cursor),
        otherwise => Err(Error::UnknownEntryType(otherwise.to_string())),
    }
}

fn parse_class(mut cursor: Cursor<&str>) -> Result<Entry, Error> {
    let mut buffer = String::with_capacity(32);
    if let 0 = cursor.read_line(&mut buffer)? {
        return Err(Error::UnexpectedEol);
    }

    Ok(Entry::Class {
        names: buffer.trim().split('\t').map(|s| s.to_string()).collect(),
    })
}

fn parse_field(mut cursor: Cursor<&str>) -> Result<Entry, Error> {
    let mut owner = Vec::with_capacity(6);
    match cursor.read_until(b'\t', &mut owner) {
        Ok(0) | Err(_) => return Err(Error::UnexpectedEof),
        _ => (),
    }
    let owner = std::str::from_utf8(&owner)?.trim().to_string();
    let ty = JavaType::parse(&mut cursor)?;
    let mut buffer = String::with_capacity(32);
    if let 0 = cursor.read_line(&mut buffer)? {
        return Err(Error::UnexpectedEol);
    }

    Ok(Entry::Field {
        owner,
        class: ty,
        names: buffer.trim().split('\t').map(|s| s.to_string()).collect(),
    })
}

fn parse_method(mut cursor: Cursor<&str>) -> Result<Entry, Error> {
    let mut owner = Vec::with_capacity(6);
    match cursor.read_until(b'\t', &mut owner) {
        Ok(0) | Err(_) => return Err(Error::UnexpectedEof),
        _ => (),
    }
    let owner = std::str::from_utf8(&owner)?.trim().to_string();
    let mut buffer = [0u8];
    if let 0 = cursor.read(&mut buffer)? {
        return Err(Error::UnexpectedEol);
    }
    let mut arguments = Vec::new();
    loop {
        let arg = match JavaType::parse(&mut cursor) {
            Ok(ty) => ty,
            Err(crate::JavaTypeError::InvalidType) => break, // Found `)`
            Err(e) => return Err(Error::JavaTypeParsing(e)),
        };
        arguments.push(arg);
    }
    let return_type = JavaType::parse(&mut cursor)?;
    let mut buffer = String::with_capacity(32);
    if let 0 = cursor.read_line(&mut buffer)? {
        return Err(Error::UnexpectedEol);
    }

    Ok(Entry::Method {
        owner,
        arguments,
        return_type,
        names: buffer.trim().split('\t').map(|s| s.to_string()).collect(),
    })
}
