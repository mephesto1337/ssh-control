use std::{
    borrow::Cow,
    io::{self, Read, Write},
    mem::size_of,
};

use nom::{
    combinator::{map, verify},
    error::context,
    multi::{length_data, many0},
    number::streaming::be_u32,
    sequence::{preceded, tuple},
};

pub trait NomError<'a>:
    nom::error::ParseError<&'a [u8]> + nom::error::ContextError<&'a [u8]>
{
}

impl<'a, E> NomError<'a> for E where
    E: nom::error::ParseError<&'a [u8]> + nom::error::ContextError<&'a [u8]>
{
}

pub trait Wire: Sized {
    fn parse<'a, E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>;

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write;
}

pub mod client;
pub mod server;
mod strings;
mod utils;
pub use client::MuxMessage;

const MUX_HELLO: u32 = 0x00000001;

#[derive(Debug)]
pub struct Extension<'a> {
    pub name: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

impl Wire for Extension<'_> {
    fn parse<'a, E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "Extension",
            map(
                tuple((<Cow<'_, str> as Wire>::parse, <Cow<'_, str> as Wire>::parse)),
                |(name, value)| Self { name, value },
            ),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.name.serialize(writer)?;
        self.value.serialize(writer)
    }
}

#[derive(Debug)]
pub struct Hello<'a> {
    pub version: u32,
    pub extensions: Vec<Extension<'a>>,
}

impl Wire for Hello<'_> {
    fn parse<'a, E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "Hello",
            preceded(
                verify(be_u32, |v| *v == MUX_HELLO),
                map(
                    tuple((be_u32, |i: &'a [u8]| {
                        if i.is_empty() {
                            Ok((i, Vec::new()))
                        } else {
                            many0(Extension::parse)(i)
                        }
                    })),
                    |(version, extensions)| Self {
                        version,
                        extensions,
                    },
                ),
            ),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        MUX_HELLO.serialize(writer)?;
        self.version.serialize(writer)?;
        for e in &self.extensions {
            e.serialize(writer)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Packet {
    buffer: Vec<u8>,
}

impl From<Vec<u8>> for Packet {
    fn from(buffer: Vec<u8>) -> Self {
        Self { buffer }
    }
}

impl Packet {
    pub fn set<T: Wire>(&mut self, val: &T) {
        const ZERO: [u8; 4] = 0u32.to_be_bytes();
        self.buffer.clear();
        self.buffer.extend_from_slice(&ZERO[..]);
        val.serialize(&mut self.buffer).unwrap();
        let size: u32 = self
            .buffer
            .len()
            .checked_sub(ZERO.len())
            .unwrap()
            .try_into()
            .expect("Buffer is over 4GB?!");
        for (i, b) in size.to_be_bytes().iter().enumerate() {
            self.buffer[i] = *b;
        }
    }

    #[allow(clippy::uninit_vec)]
    fn recv<R>(&mut self, reader: &mut R) -> io::Result<()>
    where
        R: Read,
    {
        let mut raw_size = [0u8; size_of::<u32>()];
        reader.read_exact(&mut raw_size[..])?;
        let size = u32::from_be_bytes(raw_size) as usize;
        self.buffer.clear();
        self.buffer.reserve(size);
        log::debug!("Will received {size} bytes object");
        unsafe { self.buffer.set_len(size) };
        if let Err(e) = reader.read_exact(&mut self.buffer[..]) {
            unsafe { self.buffer.set_len(0) };
            Err(e)
        } else {
            Ok(())
        }
    }

    pub fn recv_next<T, R>(&mut self, reader: &mut R) -> crate::Result<T>
    where
        T: Wire + std::fmt::Debug,
        R: Read,
    {
        self.recv(reader)?;
        let (rest, obj) = T::parse(&self.buffer[..])?;
        log::debug!("Received {obj:?}");
        assert_eq!(rest.len(), 0);
        Ok(obj)
    }
}

impl Wire for Packet {
    fn parse<'a, E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "Packet",
            map(length_data(be_u32), |buffer: &[u8]| Self {
                buffer: buffer.to_vec(),
            }),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        writer.write_all(&self.buffer[..])
    }
}
