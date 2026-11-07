//! Writing RPP through the [`Serialize`] implementation of [`Element`](crate::parser::Element)
//! and [`Child`](crate::parser::Child), so that what
//! [`parse_element`](crate::parser::parse_element) reads is written back byte for byte.
//!
//! RPP indents a line two spaces for each element it's in, and separates the tokens of a line by
//! one space. Raw text lines ([`Child::Text`](crate::parser::Child::Text)) start with `|`. Tokens
//! keep the quotes the file had, see [`Token`](crate::parser::Token).

use std::fmt;

use serde::ser::{self, Impossible, Serialize};

/// Writes `value`, an element or one of its children, as RPP, ending each line with `newline`
/// (REAPER writes `\r\n`).
pub fn to_string<T: Serialize + ?Sized>(value: &T, newline: &str) -> Result<String, Error> {
    let mut writer = Writer {
        out: String::new(),
        depth: 0,
        newline,
        separated: false,
    };
    value.serialize(&mut writer)?;
    Ok(writer.out)
}

/// A value that isn't an element, one of its children or one of their parts.
#[derive(Debug)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(message: T) -> Self {
        Self(message.to_string())
    }
}

/// The RPP being written.
struct Writer<'n> {
    out: String,
    /// How many elements the line being written is in.
    depth: usize,
    newline: &'n str,
    /// Whether the next token of the line being written follows another, and so a space.
    separated: bool,
}

impl Writer<'_> {
    /// Starts a line, indented for the elements it's in.
    fn start_line(&mut self) {
        for _ in 0..self.depth {
            self.out.push_str("  ");
        }
        self.separated = false;
    }

    fn end_line(&mut self) {
        self.out.push_str(self.newline);
    }
}

/// Fails on a part of the serde data model RPP has no place for.
fn unsupported<T>(what: &str) -> Result<T, Error> {
    Err(Error(format!("RPP has no place for {what}")))
}

impl ser::Serializer for &mut Writer<'_> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<(), Error>;

    /// A token, or an element's tag, after a space if it follows another on its line.
    fn serialize_str(self, token: &str) -> Result<(), Error> {
        if self.separated {
            self.out.push(' ');
        }
        self.out.push_str(token);
        self.separated = true;
        Ok(())
    }

    /// A [`Token`](crate::parser::Token).
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        token: &T,
    ) -> Result<(), Error> {
        token.serialize(self)
    }

    /// A [`Child`](crate::parser::Child): a line of tokens, raw text, or an element.
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        child: &T,
    ) -> Result<(), Error> {
        match variant {
            "Line" => {
                self.start_line();
                child.serialize(&mut *self)?;
                self.end_line();
            }
            "Text" => {
                self.start_line();
                self.out.push('|');
                child.serialize(&mut *self)?;
                self.end_line();
            }
            "Element" => child.serialize(self)?,
            _ => return unsupported(variant),
        }
        Ok(())
    }

    /// The tokens of a line or of an element's first line, or an element's children.
    fn serialize_seq(self, _length: Option<usize>) -> Result<Self, Error> {
        Ok(self)
    }

    /// An [`Element`](crate::parser::Element): its fields are written in order, see
    /// [`ser::SerializeStruct`].
    fn serialize_struct(self, _name: &'static str, _length: usize) -> Result<Self, Error> {
        self.start_line();
        self.out.push('<');
        Ok(self)
    }

    fn serialize_bool(self, _: bool) -> Result<(), Error> {
        unsupported("a boolean")
    }
    fn serialize_i8(self, _: i8) -> Result<(), Error> {
        unsupported("a number")
    }
    fn serialize_i16(self, _: i16) -> Result<(), Error> {
        unsupported("a number")
    }
    fn serialize_i32(self, _: i32) -> Result<(), Error> {
        unsupported("a number")
    }
    fn serialize_i64(self, _: i64) -> Result<(), Error> {
        unsupported("a number")
    }
    fn serialize_u8(self, _: u8) -> Result<(), Error> {
        unsupported("a number")
    }
    fn serialize_u16(self, _: u16) -> Result<(), Error> {
        unsupported("a number")
    }
    fn serialize_u32(self, _: u32) -> Result<(), Error> {
        unsupported("a number")
    }
    fn serialize_u64(self, _: u64) -> Result<(), Error> {
        unsupported("a number")
    }
    fn serialize_f32(self, _: f32) -> Result<(), Error> {
        unsupported("a number")
    }
    fn serialize_f64(self, _: f64) -> Result<(), Error> {
        unsupported("a number")
    }
    fn serialize_char(self, _: char) -> Result<(), Error> {
        unsupported("a character")
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), Error> {
        unsupported("bytes")
    }
    fn serialize_none(self) -> Result<(), Error> {
        unsupported("an option")
    }
    fn serialize_some<T: Serialize + ?Sized>(self, _: &T) -> Result<(), Error> {
        unsupported("an option")
    }
    fn serialize_unit(self) -> Result<(), Error> {
        unsupported("a unit")
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<(), Error> {
        unsupported(name)
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        unsupported(variant)
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Error> {
        unsupported("a tuple")
    }
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        unsupported(name)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        unsupported(variant)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Error> {
        unsupported("a map")
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        unsupported(variant)
    }
}

impl ser::SerializeSeq for &mut Writer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

/// An element is its first line, `<` with its tag and attributes, then its children one
/// element deeper, then its last line, `>`.
impl ser::SerializeStruct for &mut Writer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        field: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        match field {
            "tag" => value.serialize(&mut **self),
            "attr" => {
                value.serialize(&mut **self)?;
                self.end_line();
                self.depth += 1;
                Ok(())
            }
            "children" => value.serialize(&mut **self),
            _ => unsupported(field),
        }
    }

    fn end(self) -> Result<(), Error> {
        self.depth -= 1;
        self.start_line();
        self.out.push('>');
        self.end_line();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{parser::parse_element, ser::to_string};

    /// A project as REAPER writes it: tokens quoted with each quote character, and without
    /// quotes where they don't need them, raw text, nested elements, and CRLF line endings.
    const PROJECT: &str = concat!(
        "<REAPER_PROJECT 0.1 \"7.79/linux-x86_64\" 1794001593 0\r\n",
        "  RECORD_PATH \"\" \"\"\r\n",
        "  <NOTES 0 2\r\n",
        "    |Notes with  two spaces, ' and \"\r\n",
        "  >\r\n",
        "  <TRACK {2DACA548-6E39-0B79-60B1-15371272D92A}\r\n",
        "    NAME 'Drums \"kit\"'\r\n",
        "    AUXRECV 5 0 1 0 0 0 0 0 0 -1:U 0 -1 ''\r\n",
        "    <FXCHAIN\r\n",
        "      <VST \"VST: ReaComp (Cockos)\" reacomp.vst.so 0 `a \"b\" 'c'` 1919247213<5653544372636D72656163>\r\n",
        "        bWNlcu5e7f4AAAAAAgAAAAEAAAAAAAAAAgAAAAAAAAA8AAAAAQAAAAAAEAA=\r\n",
        "      >\r\n",
        "    >\r\n",
        "  >\r\n",
        ">\r\n",
    );

    #[test]
    fn a_project_is_written_back_as_it_was_read() {
        let project = parse_element(PROJECT).unwrap();
        assert_eq!(to_string(&project, "\r\n").unwrap(), PROJECT);
    }

    #[test]
    fn a_token_is_its_word_without_its_quotes() {
        let project = parse_element(PROJECT).unwrap();
        let words: Vec<&str> = project.attr.iter().map(|token| token.text()).collect();
        assert_eq!(words, ["0.1", "7.79/linux-x86_64", "1794001593", "0"]);
    }
}
