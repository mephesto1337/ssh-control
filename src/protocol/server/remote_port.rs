use std::io::{self, Write};

use nom::{combinator::map, error::context, number::streaming::be_u32, sequence::tuple};

use crate::protocol::{server::MuxResponse, NomError, Wire};

#[derive(Debug)]
pub struct RemotePort {
    pub client_request_id: u32,
    pub allocated_remote_listen_port: u32,
}

impl<'a> Wire<'a> for RemotePort {
    fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "RemotePort",
            map(
                tuple((be_u32, be_u32)),
                |(client_request_id, allocated_remote_listen_port)| Self {
                    client_request_id,
                    allocated_remote_listen_port,
                },
            ),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.client_request_id.serialize(writer)?;
        self.allocated_remote_listen_port.serialize(writer)
    }
}

impl RemotePort {
    pub fn into_owned(self) -> Self {
        self
    }
}

impl From<RemotePort> for MuxResponse<'_> {
    fn from(value: RemotePort) -> Self {
        Self::RemotePort(value)
    }
}
