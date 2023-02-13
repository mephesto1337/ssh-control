use std::io::{self, Write};

use nom::{combinator::map, error::context, number::streaming::be_u32, sequence::tuple};

use crate::protocol::{server::MuxResponse, NomError, Wire};

#[derive(Debug)]
pub struct ExitMessage {
    pub session_id: u32,
    pub exit_value: u32,
}

impl<'a> Wire<'a> for ExitMessage {
    fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        log::debug!("ExitMessage::parse: {}", crate::error::RawBytes(input));
        context(
            "ExitMessage",
            map(tuple((be_u32, be_u32)), |(session_id, exit_value)| Self {
                session_id,
                exit_value,
            }),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.session_id.serialize(writer)?;
        self.exit_value.serialize(writer)
    }
}

impl ExitMessage {
    pub fn into_owned(self) -> Self {
        self
    }
}

impl From<ExitMessage> for MuxResponse<'_> {
    fn from(value: ExitMessage) -> Self {
        Self::ExitMessage(value)
    }
}
