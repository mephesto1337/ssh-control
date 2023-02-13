use std::io::{self, Write};

use nom::{combinator::map, error::context, number::streaming::be_u32, sequence::tuple};

use crate::protocol::{server::MuxResponse, NomError, Wire};

#[derive(Debug)]
pub struct SessionOpened {
    pub client_request_id: u32,
    pub session_id: u32,
}

impl<'a> Wire<'a> for SessionOpened {
    fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "SessionOpened",
            map(
                tuple((be_u32, be_u32)),
                |(client_request_id, session_id)| Self {
                    client_request_id,
                    session_id,
                },
            ),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.client_request_id.serialize(writer)?;
        self.session_id.serialize(writer)
    }
}

impl SessionOpened {
    pub fn into_owned(self) -> Self {
        self
    }
}

impl From<SessionOpened> for MuxResponse<'_> {
    fn from(value: SessionOpened) -> Self {
        Self::SessionOpened(value)
    }
}
