use std::io::{self, Write};

use nom::{combinator::map, error::context, number::streaming::be_u32};

use crate::protocol::{server::MuxResponse, NomError, Wire};

#[derive(Debug)]
pub struct TtyAllocFail {
    pub session_id: u32,
}

impl<'a> Wire<'a> for TtyAllocFail {
    fn parse<E>(input: &'a [u8]) -> nom::IResult<&'a [u8], Self, E>
    where
        E: NomError<'a>,
    {
        context(
            "TtyAllocFail",
            map(be_u32, |session_id| Self { session_id }),
        )(input)
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        self.session_id.serialize(writer)
    }
}

impl TtyAllocFail {
    pub fn into_owned(self) -> Self {
        self
    }
}

impl From<TtyAllocFail> for MuxResponse<'_> {
    fn from(value: TtyAllocFail) -> Self {
        Self::TtyAllocFail(value)
    }
}
