use std::io::{self, Write};

use nom::{
    combinator::{map, map_opt, verify},
    error::context,
    number::streaming::be_u32,
};

use crate::protocol::{NomError, Wire};

mod alive_check;
mod close_fwd;
mod new_session;
mod new_stdio_fwd;
mod open_fwd;
mod stop_listening;
mod terminate;

pub use alive_check::AliveCheck;
pub use close_fwd::CloseFwd;
pub use new_session::NewSession;
pub use new_stdio_fwd::NewStdioFwd;
pub use open_fwd::OpenFwd;
pub use stop_listening::StopListening;
pub use terminate::Terminate;

const NEW_SESSION: u32 = 0x10000002;
const ALIVE_CHECK: u32 = 0x10000004;
const TERMINATE: u32 = 0x10000005;
const OPEN_FWD: u32 = 0x10000006;
const CLOSE_FWD: u32 = 0x10000007;
const NEW_STDIO_FWD: u32 = 0x10000008;
const STOP_LISTENING: u32 = 0x10000009;

const FWD_LOCAL: u32 = 1;
const FWD_REMOTE: u32 = 2;
const FWD_DYNAMIC: u32 = 3;

const LISTEN_TYPE_UNIX: u32 = -2i32 as u32;

#[derive(Debug)]
#[repr(u32)]
pub enum ForwardingType {
    Local,
    Remote,
    Dynamic,
}

impl<'a> Wire<'a> for ForwardingType {
    fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "ForwardingType",
            map_opt(be_u32, |v| match v {
                FWD_LOCAL => Some(Self::Local),
                FWD_REMOTE => Some(Self::Remote),
                FWD_DYNAMIC => Some(Self::Dynamic),
                _ => None,
            }),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        match self {
            Self::Local => FWD_LOCAL.serialize(writer),
            Self::Remote => FWD_REMOTE.serialize(writer),
            Self::Dynamic => FWD_DYNAMIC.serialize(writer),
        }
    }
}

#[derive(Debug)]
pub enum ListenType {
    Inet(u16),
    Unix,
}

impl<'a> Wire<'a> for ListenType {
    fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "ListenType",
            map_opt(be_u32, |port| match port {
                LISTEN_TYPE_UNIX => Some(Self::Unix),
                _ => port.try_into().ok().map(Self::Inet),
            }),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        match self {
            ListenType::Inet(port) => (*port as u32).serialize(writer),
            ListenType::Unix => LISTEN_TYPE_UNIX.serialize(writer),
        }
    }
}

#[derive(Debug)]
pub enum MuxMessage<'a> {
    NewSession(new_session::NewSession<'a>),
    AliveCheck(alive_check::AliveCheck),
    Terminate(terminate::Terminate),
    OpenFwd(open_fwd::OpenFwd<'a>),
    CloseFwd(close_fwd::CloseFwd<'a>),
    NewStdioFwd(new_stdio_fwd::NewStdioFwd<'a>),
    StopListening(stop_listening::StopListening),
}

impl<'a> Wire<'a> for MuxMessage<'a> {
    fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
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
            NEW_SESSION => map(new_session::NewSession::parse, Self::NewSession)(rest),
            ALIVE_CHECK => map(alive_check::AliveCheck::parse, Self::AliveCheck)(rest),
            TERMINATE => map(terminate::Terminate::parse, Self::Terminate)(rest),
            OPEN_FWD => map(open_fwd::OpenFwd::parse, Self::OpenFwd)(rest),
            CLOSE_FWD => map(close_fwd::CloseFwd::parse, Self::CloseFwd)(rest),
            NEW_STDIO_FWD => map(new_stdio_fwd::NewStdioFwd::parse, Self::NewStdioFwd)(rest),
            STOP_LISTENING => map(stop_listening::StopListening::parse, Self::StopListening)(rest),
            _ => unreachable!(),
        }
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        match self {
            Self::NewSession(body) => {
                NEW_SESSION.serialize(writer)?;
                body.serialize(writer)
            }
            Self::AliveCheck(body) => {
                ALIVE_CHECK.serialize(writer)?;
                body.serialize(writer)
            }
            Self::Terminate(body) => {
                TERMINATE.serialize(writer)?;
                body.serialize(writer)
            }
            Self::OpenFwd(body) => {
                OPEN_FWD.serialize(writer)?;
                body.serialize(writer)
            }
            Self::CloseFwd(body) => {
                CLOSE_FWD.serialize(writer)?;
                body.serialize(writer)
            }
            Self::NewStdioFwd(body) => {
                NEW_STDIO_FWD.serialize(writer)?;
                body.serialize(writer)
            }
            Self::StopListening(body) => {
                STOP_LISTENING.serialize(writer)?;
                body.serialize(writer)
            }
        }
    }
}
impl<'a> MuxMessage<'a> {
    pub fn into_owned(self) -> MuxMessage<'static> {
        match self {
            Self::NewSession(body) => MuxMessage::NewSession(body.into_owned()),
            Self::AliveCheck(body) => MuxMessage::AliveCheck(body.into_owned()),
            Self::Terminate(body) => MuxMessage::Terminate(body.into_owned()),
            Self::OpenFwd(body) => MuxMessage::OpenFwd(body.into_owned()),
            Self::CloseFwd(body) => MuxMessage::CloseFwd(body.into_owned()),
            Self::NewStdioFwd(body) => MuxMessage::NewStdioFwd(body.into_owned()),
            Self::StopListening(body) => MuxMessage::StopListening(body.into_owned()),
        }
    }
}

impl MuxMessage<'_> {
    pub fn set_request_id(&mut self, request_id: u32) {
        match self {
            Self::NewSession(ref mut body) => {
                body.request_id = request_id;
            }
            Self::AliveCheck(ref mut body) => {
                body.request_id = request_id;
            }
            Self::Terminate(ref mut body) => {
                body.request_id = request_id;
            }
            Self::OpenFwd(ref mut body) => {
                body.request_id = request_id;
            }
            Self::CloseFwd(ref mut body) => {
                body.request_id = request_id;
            }
            Self::NewStdioFwd(ref mut body) => {
                body.request_id = request_id;
            }
            Self::StopListening(ref mut body) => {
                body.request_id = request_id;
            }
        }
    }

    pub fn get_request_id(&self) -> u32 {
        match self {
            Self::NewSession(ref body) => body.request_id,
            Self::AliveCheck(ref body) => body.request_id,
            Self::Terminate(ref body) => body.request_id,
            Self::OpenFwd(ref body) => body.request_id,
            Self::CloseFwd(ref body) => body.request_id,
            Self::NewStdioFwd(ref body) => body.request_id,
            Self::StopListening(ref body) => body.request_id,
        }
    }
}
