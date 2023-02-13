use std::io::{self, Write};

use nom::{combinator::map, error::context, number::streaming::be_u32};

use crate::protocol::{server::MuxResponse, NomError, Wire};

#[derive(Debug)]
pub struct Ok {
    pub client_request_id: u32,
}

impl<'a> Wire<'a> for Ok {
    fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "Ok",
            map(be_u32, |client_request_id| Self { client_request_id }),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.client_request_id.serialize(writer)
    }
}

impl Ok {
    pub fn into_owned(self) -> Self
    where
        Self: 'static,
    {
        self
    }
}

impl From<Ok> for MuxResponse<'_> {
    fn from(value: Ok) -> Self {
        Self::Ok(value)
    }
}
