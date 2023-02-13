use std::io::{self, Write};

use nom::{combinator::map, error::context, number::streaming::be_u32, sequence::tuple};

use crate::protocol::{server::MuxResponse, NomError, Wire};

#[derive(Debug)]
pub struct Alive {
    pub client_request_id: u32,
    pub server_pid: u32,
}

impl Alive {
    pub fn into_owned(self) -> Self {
        self
    }
}

impl<'a> Wire<'a> for Alive {
    fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "Alive",
            map(
                tuple((be_u32, be_u32)),
                |(client_request_id, server_pid)| Self {
                    client_request_id,
                    server_pid,
                },
            ),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.client_request_id.serialize(writer)?;
        self.server_pid.serialize(writer)
    }
}

impl From<Alive> for MuxResponse<'_> {
    fn from(value: Alive) -> Self {
        Self::Alive(value)
    }
}
