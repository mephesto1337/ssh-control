use std::io::{self, Write};

use nom::{combinator::map, multi::many0, number::streaming::be_u32};

use crate::protocol::{NomError, Wire};

macro_rules! impl_wire_nums {
    ($type:ty, $parser:ident) => {
        impl<'a> Wire<'a> for $type {
            fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
            where
                E: NomError<'a>,
            {
                $parser(input)
            }

            fn serialize<W>(&self, writer: &mut W) -> ::std::io::Result<()>
            where
                W: ::std::io::Write,
            {
                writer.write_all(&self.to_be_bytes()[..])
            }
        }
    };
}
impl_wire_nums!(u32, be_u32);

impl<'a> Wire<'a> for bool {
    fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        map(be_u32, |v| v != 0)(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        let val = if *self { 1 } else { 0 };
        val.serialize(writer)
    }
}

pub fn many<'a, O, E, F>(mut f: F) -> impl FnMut(&'a [u8]) -> nom::IResult<&'a [u8], Vec<O>, E>
where
    F: nom::Parser<&'a [u8], O, E>,
    E: NomError<'a>,
{
    move |input: &'a [u8]| {
        if input.is_empty() {
            Ok((input, Vec::new()))
        } else {
            many0(|i| f.parse(i))(input)
        }
    }
}
