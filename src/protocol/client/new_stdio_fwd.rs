use std::{
    borrow::Cow,
    io::{self, Write},
};

use nom::{combinator::map, error::context, number::streaming::be_u32, sequence::tuple};

use crate::protocol::{
    client::{MuxMessage, Port},
    NomError, Wire,
};

#[derive(Debug)]
pub struct NewStdioFwd<'a> {
    pub request_id: u32,
    pub connect_host: Cow<'a, str>,
    pub connect_port: Port,
}

impl Wire for NewStdioFwd<'_> {
    fn parse<'a, E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "NewStdioFwd",
            map(
                tuple((
                    be_u32,
                    <Cow<'_, str> as Wire>::parse,
                    <Cow<'_, str> as Wire>::parse,
                    Port::parse,
                )),
                |(request_id, reserved, connect_host, connect_port)| {
                    if !reserved.is_empty() {
                        log::warn!("reserved is not empty: {reserved:?}");
                    }
                    Self {
                        request_id,
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
        Cow::Borrowed("").serialize(writer)?;
        self.connect_host.serialize(writer)?;
        self.connect_port.serialize(writer)
    }
}

impl<'a> From<NewStdioFwd<'a>> for MuxMessage<'a> {
    fn from(value: NewStdioFwd<'a>) -> Self {
        Self::NewStdioFwd(value)
    }
}
