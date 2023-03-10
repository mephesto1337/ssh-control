use std::{
    borrow::Cow,
    io::{self, Write},
};

use nom::{combinator::map, error::context, number::streaming::be_u32, sequence::tuple};

use crate::protocol::{
    client::{ForwardingType, MuxMessage, Port},
    NomError, Wire,
};

#[derive(Debug)]
pub struct CloseFwd<'a> {
    pub request_id: u32,
    pub forwarding_type: ForwardingType,
    pub listen_host: Cow<'a, str>,
    pub listen_port: Port,
    pub connect_host: Cow<'a, str>,
    pub connect_port: Port,
}

impl<'a> Wire<'a> for CloseFwd<'a> {
    fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "CloseFwd",
            map(
                tuple((
                    be_u32,
                    ForwardingType::parse,
                    <Cow<'_, str> as Wire>::parse,
                    Port::parse,
                    <Cow<'_, str> as Wire>::parse,
                    Port::parse,
                )),
                |(
                    request_id,
                    forwarding_type,
                    listen_host,
                    listen_port,
                    connect_host,
                    connect_port,
                )| {
                    Self {
                        request_id,
                        forwarding_type,
                        listen_host,
                        listen_port,
                        connect_host,
                        connect_port,
                    }
                },
            ),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.request_id.serialize(writer)?;
        self.forwarding_type.serialize(writer)?;
        self.listen_host.serialize(writer)?;
        self.listen_port.serialize(writer)?;
        self.connect_host.serialize(writer)?;
        self.connect_port.serialize(writer)
    }
}

impl<'a> CloseFwd<'a> {
    pub fn into_owned(self) -> CloseFwd<'static> {
        CloseFwd {
            request_id: self.request_id,
            forwarding_type: self.forwarding_type,
            listen_host: Cow::Owned(self.listen_host.into_owned()),
            listen_port: self.listen_port,
            connect_host: Cow::Owned(self.connect_host.into_owned()),
            connect_port: self.connect_port,
        }
    }
}

impl<'a> From<CloseFwd<'a>> for MuxMessage<'a> {
    fn from(value: CloseFwd<'a>) -> Self {
        Self::CloseFwd(value)
    }
}
