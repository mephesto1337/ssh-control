use std::{
    borrow::Cow,
    io::{self, Write},
};

use nom::{combinator::map, error::context, number::streaming::be_u32, sequence::tuple};

use crate::protocol::{server::MuxResponse, NomError, Wire};

#[derive(Debug)]
pub struct Failure<'a> {
    pub client_request_id: u32,
    pub reason: Cow<'a, str>,
}

impl<'a> Wire<'a> for Failure<'a> {
    fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "Failure",
            map(
                tuple((be_u32, <Cow<'_, str> as Wire>::parse)),
                |(client_request_id, reason)| Self {
                    client_request_id,
                    reason,
                },
            ),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.client_request_id.serialize(writer)?;
        self.reason.serialize(writer)
    }
}

impl<'a> Failure<'a> {
    pub fn into_owned(self) -> Failure<'static> {
        Failure {
            client_request_id: self.client_request_id,
            reason: Cow::Owned(self.reason.into_owned()),
        }
    }
}

impl<'a> From<Failure<'a>> for MuxResponse<'a> {
    fn from(value: Failure<'a>) -> Self {
        Self::Failure(value)
    }
}
