use std::io::{self, Write};

use nom::{
    combinator::{map, verify},
    error::context,
    number::streaming::be_u32,
};

use crate::protocol::{NomError, Wire};

mod alive;
mod exit_message;
mod failure;
mod ok;
mod permission_denied;
mod remote_port;
mod session_opened;
mod tty_alloc_fail;

pub use alive::Alive;
pub use exit_message::ExitMessage;
pub use failure::Failure;
pub use ok::Ok;
pub use permission_denied::PermissionDenied;
pub use remote_port::RemotePort;
pub use session_opened::SessionOpened;
pub use tty_alloc_fail::TtyAllocFail;

const OK: u32 = 0x80000001;
const PERMISSION_DENIED: u32 = 0x80000002;
const FAILURE: u32 = 0x80000003;
const EXIT_MESSAGE: u32 = 0x80000004;
const ALIVE: u32 = 0x80000005;
const SESSION_OPENED: u32 = 0x80000006;
const REMOTE_PORT: u32 = 0x80000007;
const TTY_ALLOC_FAIL: u32 = 0x80000008;

#[derive(Debug)]
pub enum MuxResponse<'a> {
    Ok(ok::Ok),
    PermissionDenied(permission_denied::PermissionDenied<'a>),
    Failure(failure::Failure<'a>),
    ExitMessage(exit_message::ExitMessage),
    Alive(alive::Alive),
    SessionOpened(session_opened::SessionOpened),
    RemotePort(remote_port::RemotePort),
    TtyAllocFail(tty_alloc_fail::TtyAllocFail),
}

impl Wire for MuxResponse<'_> {
    fn parse<'a, E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        let (rest, r#type) = context(
            "MuxResponse",
            verify(be_u32, |v| {
                matches!(
                    *v,
                    OK | PERMISSION_DENIED
                        | FAILURE
                        | EXIT_MESSAGE
                        | ALIVE
                        | SESSION_OPENED
                        | REMOTE_PORT
                        | TTY_ALLOC_FAIL
                )
            }),
        )(input)?;

        match r#type {
            OK => map(ok::Ok::parse, Self::Ok)(rest),
            PERMISSION_DENIED => map(
                permission_denied::PermissionDenied::parse,
                Self::PermissionDenied,
            )(rest),
            FAILURE => map(failure::Failure::parse, Self::Failure)(rest),
            EXIT_MESSAGE => map(exit_message::ExitMessage::parse, Self::ExitMessage)(rest),
            ALIVE => map(alive::Alive::parse, Self::Alive)(rest),
            SESSION_OPENED => map(session_opened::SessionOpened::parse, Self::SessionOpened)(rest),
            REMOTE_PORT => map(remote_port::RemotePort::parse, Self::RemotePort)(rest),
            TTY_ALLOC_FAIL => map(tty_alloc_fail::TtyAllocFail::parse, Self::TtyAllocFail)(rest),
            _ => unreachable!(),
        }
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        match self {
            Self::Ok(ref body) => {
                OK.serialize(writer)?;
                body.serialize(writer)
            }
            Self::PermissionDenied(ref body) => {
                PERMISSION_DENIED.serialize(writer)?;
                body.serialize(writer)
            }
            Self::Failure(ref body) => {
                FAILURE.serialize(writer)?;
                body.serialize(writer)
            }
            Self::ExitMessage(ref body) => {
                EXIT_MESSAGE.serialize(writer)?;
                body.serialize(writer)
            }
            Self::Alive(ref body) => {
                ALIVE.serialize(writer)?;
                body.serialize(writer)
            }
            Self::SessionOpened(ref body) => {
                SESSION_OPENED.serialize(writer)?;
                body.serialize(writer)
            }
            Self::RemotePort(ref body) => {
                REMOTE_PORT.serialize(writer)?;
                body.serialize(writer)
            }
            Self::TtyAllocFail(ref body) => {
                TTY_ALLOC_FAIL.serialize(writer)?;
                body.serialize(writer)
            }
        }
    }
}

impl<'a> MuxResponse<'a> {
    pub fn get_request_id(&self) -> Option<u32> {
        match self {
            Self::Ok(ref body) => Some(body.client_request_id),
            Self::PermissionDenied(ref body) => Some(body.client_request_id),
            Self::Failure(ref body) => Some(body.client_request_id),
            Self::ExitMessage(_) => None,
            Self::Alive(ref body) => Some(body.client_request_id),
            Self::SessionOpened(ref body) => Some(body.client_request_id),
            Self::RemotePort(ref body) => Some(body.client_request_id),
            Self::TtyAllocFail(_) => None,
        }
    }
}

macro_rules! impl_from_mux_response {
    ($variant:ident, $type:ty) => {
        impl From<MuxResponse<'_>> for $crate::Result<$type> {
            fn from(value: MuxResponse<'_>) -> Self {
                match value {
                    MuxResponse::PermissionDenied(pd) => {
                        Err($crate::Error::PermissionDenied(pd.reason.into_owned()))
                    }
                    MuxResponse::Failure(f) => {
                        Err($crate::Error::PermissionDenied(f.reason.into_owned()))
                    }
                    MuxResponse::$variant(val) => Ok(val),
                    _ => Err($crate::Error::InvalidPacket {
                        description: format!("{value:?}").into(),
                    }),
                }
            }
        }
    };
}

impl_from_mux_response!(Alive, Alive);
impl_from_mux_response!(ExitMessage, ExitMessage);
impl_from_mux_response!(Ok, Ok);
impl_from_mux_response!(RemotePort, RemotePort);
impl_from_mux_response!(SessionOpened, SessionOpened);
