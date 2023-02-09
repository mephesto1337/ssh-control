use std::{
    borrow::Cow,
    io::{self, Write},
};

use nom::{
    bytes::streaming::tag,
    combinator::{map, verify},
    error::context,
    number::streaming::be_u32,
    sequence::{terminated, tuple},
};

use crate::protocol::{utils::many, NomError, Wire};

const NEW_SESSION: u32 = 0x10000002;
const ALIVE_CHECK: u32 = 0x10000004;
const TERMINATE: u32 = 0x10000005;
const OPEN_FWD: u32 = 0x10000006;
const CLOSE_FWD: u32 = 0x10000007;
const NEW_STDIO_FWD: u32 = 0x10000008;
const STOP_LISTENING: u32 = 0x10000009;

#[derive(Debug)]
#[allow(dead_code)]
pub enum MuxMessage<'a> {
    NewSession(NewSession<'a>),
    AliveCheck(AliveCheck),
    Terminate,
    OpenFwd,
    CloseFwd,
    NewStdioFwd,
    StopListening,
}

impl Wire for MuxMessage<'_> {
    fn parse<'a, E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        let (rest, r#type) = context(
            "MuxMessage",
            verify(be_u32, |t| {
                matches!(
                    *t,
                    NEW_SESSION
                        | ALIVE_CHECK
                        | TERMINATE
                        | OPEN_FWD
                        | CLOSE_FWD
                        | NEW_STDIO_FWD
                        | STOP_LISTENING
                )
            }),
        )(input)?;
        match r#type {
            NEW_SESSION => map(NewSession::parse, Self::NewSession)(rest),
            ALIVE_CHECK => map(AliveCheck::parse, Self::AliveCheck)(rest),
            TERMINATE => todo!(),
            OPEN_FWD => todo!(),
            CLOSE_FWD => todo!(),
            NEW_STDIO_FWD => todo!(),
            STOP_LISTENING => todo!(),
            _ => unreachable!(),
        }
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        match self {
            Self::NewSession(n) => {
                NEW_SESSION.serialize(writer)?;
                n.serialize(writer)
            }
            Self::AliveCheck(a) => {
                ALIVE_CHECK.serialize(writer)?;
                a.serialize(writer)
            }
            Self::Terminate => todo!(),
            Self::OpenFwd => todo!(),
            Self::CloseFwd => todo!(),
            Self::NewStdioFwd => todo!(),
            Self::StopListening => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct NewSession<'a> {
    pub request_id: u32,
    pub want_tty: bool,
    pub want_x11_forwarding: bool,
    pub subsystem: bool,
    pub escape_char: u32,
    pub terminal_type: Cow<'a, str>,
    pub command: Cow<'a, str>,
    pub environment: Vec<Cow<'a, str>>,
}

impl Wire for NewSession<'_> {
    fn parse<'a, E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
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
        self.subsystem.serialize(writer)?;
        self.escape_char.serialize(writer)?;
        self.terminal_type.serialize(writer)?;
        self.command.serialize(writer)?;
        for e in &self.environment {
            e.serialize(writer)?;
        }
        writer.write_all(&[0][..])
    }
}

#[derive(Debug)]
pub struct AliveCheck {
    pub request_id: u32,
}

impl Wire for AliveCheck {
    fn parse<'a, E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context("AliveCheck", map(be_u32, |request_id| Self { request_id }))(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.request_id.serialize(writer)
    }
}
