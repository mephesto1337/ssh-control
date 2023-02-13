use std::io::{self, Write};

use nom::{combinator::map, error::context, number::streaming::be_u32};

use crate::protocol::{client::MuxMessage, NomError, Wire};

#[derive(Debug)]
pub struct StopListening {
    pub request_id: u32,
}

impl<'a> Wire<'a> for StopListening {
    fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "StopListening",
            map(be_u32, |request_id| Self { request_id }),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.request_id.serialize(writer)
    }
}

impl StopListening {
    pub fn into_owned(self) -> Self {
        self
    }
}

impl From<StopListening> for MuxMessage<'_> {
    fn from(value: StopListening) -> Self {
        Self::StopListening(value)
    }
}
