use std::{
    borrow::Cow,
    io::{self, Write},
};

use nom::{
    bytes::streaming::tag,
    combinator::map,
    error::context,
    number::streaming::be_u32,
    sequence::{terminated, tuple},
};

use crate::protocol::{client::MuxMessage, utils::many, NomError, Wire};

#[derive(Debug)]
pub struct NewSession<'a> {
    pub request_id: u32,
    pub want_tty: bool,
    pub want_x11_forwarding: bool,
    pub want_agent: bool,
    pub subsystem: bool,
    pub escape_char: u32,
    pub terminal_type: Cow<'a, str>,
    pub command: Cow<'a, str>,
    pub environment: Vec<Cow<'a, str>>,
}

impl<'a> Wire<'a> for NewSession<'a> {
    fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "NewSession",
            map(
                tuple((
                    be_u32,
                    <Cow<'_, str> as Wire>::parse,
                    map(be_u32, |v| v != 0),
                    map(be_u32, |v| v != 0),
                    map(be_u32, |v| v != 0),
                    map(be_u32, |v| v != 0),
                    be_u32,
                    <Cow<'_, str> as Wire>::parse,
                    <Cow<'_, str> as Wire>::parse,
                    terminated(many(<Cow<'_, str> as Wire>::parse), tag(b"\0")),
                )),
                |(
                    request_id,
                    reserved,
                    want_tty,
                    want_x11_forwarding,
                    want_agent,
                    subsystem,
                    escape_char,
                    terminal_type,
                    command,
                    environment,
                )| {
                    if !reserved.is_empty() {
                        log::warn!("Reserved string is not empty: {reserved:?}");
                    }
                    Self {
                        request_id,
                        want_tty,
                        want_x11_forwarding,
                        want_agent,
                        subsystem,
                        escape_char,
                        terminal_type,
                        command,
                        environment,
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
        self.want_tty.serialize(writer)?;
        self.want_x11_forwarding.serialize(writer)?;
        self.want_agent.serialize(writer)?;
        self.subsystem.serialize(writer)?;
        self.escape_char.serialize(writer)?;
        self.terminal_type.serialize(writer)?;
        self.command.serialize(writer)?;
        log::debug!("environment: {:?}", &self.environment);
        for e in &self.environment {
            e.serialize(writer)?;
        }
        Ok(())
        // writer.write_all(&[0u8][..])
    }
}

impl<'a> NewSession<'a> {
    pub fn into_owned(self) -> NewSession<'static> {
        NewSession {
            request_id: self.request_id,
            want_tty: self.want_tty,
            want_x11_forwarding: self.want_x11_forwarding,
            want_agent: self.want_agent,
            subsystem: self.subsystem,
            escape_char: self.escape_char,
            terminal_type: Cow::Owned(self.terminal_type.into_owned()),
            command: Cow::Owned(self.command.into_owned()),
            environment: self
                .environment
                .into_iter()
                .map(|e| Cow::Owned(Cow::into_owned(e)))
                .collect(),
        }
    }
}

impl<'a> From<NewSession<'a>> for MuxMessage<'a> {
    fn from(value: NewSession<'a>) -> Self {
        Self::NewSession(value)
    }
}
