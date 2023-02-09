use std::io::{self, Write};

use nom::{
    combinator::{map, verify},
    error::context,
    number::streaming::be_u32,
    sequence::tuple,
};

use crate::protocol::{NomError, Wire};

// const OK: u32 = 0x80000001;
// const PERMISSION_DENIED: u32 = 0x80000002;
// const FAILURE: u32 = 0x80000003;
// const EXIT_MESSAGE: u32 = 0x80000004;
const ALIVE: u32 = 0x80000005;
const SESSION_OPENED: u32 = 0x80000006;
// const REMOTE_PORT: u32 = 0x80000007;
// const TTY_ALLOC_FAIL: u32 = 0x80000008;

#[derive(Debug)]
#[non_exhaustive]
pub enum MuxResponse {
    SessionOpened(SessionOpened),
    Alive(Alive),
}

impl Wire for MuxResponse {
    fn parse<'a, E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        let (rest, r#type) = context(
            "MuxResponse",
            verify(be_u32, |t| matches!(*t, SESSION_OPENED | ALIVE)),
        )(input)?;

        match r#type {
            SESSION_OPENED => map(SessionOpened::parse, Self::SessionOpened)(rest),
            ALIVE => map(Alive::parse, Self::Alive)(rest),
            _ => unreachable!(),
        }
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        match self {
            Self::SessionOpened(so) => {
                SESSION_OPENED.serialize(writer)?;
                so.serialize(writer)
            }
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct SessionOpened {
    pub request_id: u32,
    pub session_id: u32,
}

impl Wire for SessionOpened {
    fn parse<'a, E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "SessionOpened",
            map(tuple((be_u32, be_u32)), |(request_id, session_id)| Self {
                request_id,
                session_id,
            }),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.request_id.serialize(writer)?;
        self.session_id.serialize(writer)
    }
}

#[derive(Debug)]
pub struct Alive {
    pub request_id: u32,
    pub server_pid: u32,
}

impl Wire for Alive {
    fn parse<'a, E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "Alive",
            map(tuple((be_u32, be_u32)), |(request_id, server_pid)| Self {
                request_id,
                server_pid,
            }),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.request_id.serialize(writer)?;
        self.server_pid.serialize(writer)
    }
}
