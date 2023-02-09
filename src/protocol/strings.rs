use std::{
    borrow::Cow,
    io::{self, Write},
};

use nom::{
    combinator::{map, map_opt, rest},
    error::context,
    multi::length_value,
    number::streaming::be_u32,
};

use crate::protocol::{NomError, Wire};

impl Wire for String {
    fn parse<'a, E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "string",
            length_value(
                be_u32,
                map_opt(rest, |bytes| {
                    std::str::from_utf8(bytes).ok().map(String::from)
                }),
            ),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        let size = self.len() as u32;
        size.serialize(writer)?;
        writer.write_all(self.as_bytes())
    }
}

impl Wire for Cow<'_, str> {
    fn parse<'a, E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        map(<String as Wire>::parse, Cow::from)(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        match self {
            Cow::Borrowed(b) => {
                let size = b.len() as u32;
                size.serialize(writer)?;
                writer.write_all(b.as_bytes())
            }
            Cow::Owned(o) => o.serialize(writer),
        }
    }
}
