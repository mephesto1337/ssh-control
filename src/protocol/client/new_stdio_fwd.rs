use std::{
    borrow::Cow,
    io::{self, Write},
};

use nom::{combinator::map, error::context, number::streaming::be_u32, sequence::tuple};

use crate::protocol::{
    client::{ListenType, MuxMessage},
    NomError, Wire,
};

#[derive(Debug)]
pub struct NewStdioFwd<'a> {
    pub request_id: u32,
    pub connect_host: Cow<'a, str>,
    pub connect_port: ListenType,
}

impl<'a> Wire<'a> for NewStdioFwd<'a> {
    fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
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
                    ListenType::parse,
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

impl<'a> NewStdioFwd<'a> {
    pub fn into_owned(self) -> NewStdioFwd<'static> {
        NewStdioFwd {
            request_id: self.request_id,
            connect_host: Cow::Owned(self.connect_host.into_owned()),
            connect_port: self.connect_port,
        }
    }
}

impl<'a> From<NewStdioFwd<'a>> for MuxMessage<'a> {
    fn from(value: NewStdioFwd<'a>) -> Self {
        Self::NewStdioFwd(value)
    }
}
