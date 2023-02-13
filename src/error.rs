use nom::error::{VerboseError, VerboseErrorKind};

use std::{
    borrow::Cow,
    fmt::{self, Write},
    io,
};

pub struct RawBytes<I>(pub I);

impl<I> fmt::Display for RawBytes<I>
where
    I: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn show_string(s: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for c in s.bytes() {
                match c {
                    b'\n' => f.write_str("\\n")?,
                    b'\r' => f.write_str("\\r")?,
                    b'\t' => f.write_str("\\t")?,
                    _ => {
                        if c.is_ascii_alphanumeric() || c.is_ascii_punctuation() || c == b' ' {
                            f.write_char(c as char)?;
                        } else {
                            write!(f, "\\x{c:02x}")?;
                        }
                    }
                }
            }
            Ok(())
        }

        let mut data = self.0.as_ref();
        while !data.is_empty() {
            match std::str::from_utf8(data) {
                Ok(v) => {
                    return show_string(v, f);
                }
                Err(e) => {
                    if e.valid_up_to() > 0 {
                        let s = unsafe { std::str::from_utf8_unchecked(&data[..e.valid_up_to()]) };
                        show_string(s, f)?;
                    }
                    write!(f, "\\{:02x}", data[e.valid_up_to()])?;
                    data = &data[e.valid_up_to() + 1..];
                }
            }
        }
        Ok(())
    }
}

impl<I> fmt::Debug for RawBytes<I>
where
    I: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("0x")?;
        for b in self.0.as_ref() {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    /// Underlying I/O error
    IO(io::Error),

    /// Parsing error
    Parsing(VerboseError<RawBytes<Vec<u8>>>),

    /// Incomplete buffer
    Incomplete(nom::Needed),

    /// Unsupported version
    UnsupportedVersion(u32),

    /// Invalid Packet
    InvalidPacket { description: Cow<'static, str> },

    /// Bad request ID
    InvalidResponseID {
        expected: Option<u32>,
        received: Option<u32>,
    },

    /// Permission denied
    PermissionDenied(String),

    /// Failure
    Failure(String),

    /// TTY allocation failed,
    TtyAllocFailed,
}
pub type Result<T> = ::std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(ref e) => fmt::Display::fmt(e, f),
            Self::Parsing(ref vb) => {
                writeln!(f, "Parsing error:")?;
                for (i, (input, kind)) in vb.errors.iter().enumerate() {
                    match kind {
                        VerboseErrorKind::Context(s) => writeln!(f, "{i}: {s} at {input}")?,
                        VerboseErrorKind::Nom(k) => writeln!(f, "{i}: {k:?} at {input}")?,
                        VerboseErrorKind::Char(c) => {
                            writeln!(f, "{i}: unexpected char {c} at {input}")?
                        }
                    }
                }
                Ok(())
            }
            Self::Incomplete(ref n) => match n {
                nom::Needed::Unknown => f.write_str("More bytes needed"),
                nom::Needed::Size(b) => write!(f, "At least {b} more bytes needed"),
            },
            Self::UnsupportedVersion(version) => {
                write!(f, "Remote uses incompatible version {version}")
            }
            Self::InvalidPacket { description } => {
                write!(f, "Received an invalid packet: {description}")
            }
            Self::InvalidResponseID { expected, received } => match (expected, received) {
                (Some(exp), Some(rec)) => write!(f, "Expect ID 0x{exp:x}, received 0x{rec:x}"),
                (Some(exp), None) => write!(f, "Expect ID 0x{exp:x}, received none"),
                (None, Some(rec)) => write!(f, "Expect no ID, received 0x{rec:x}"),
                _ => unreachable!(),
            },
            Self::PermissionDenied(ref reason) => {
                write!(f, "Remote operation not permietted: {reason}")
            }
            Self::Failure(ref reason) => {
                write!(f, "Remote operation failed: {reason}")
            }
            Self::TtyAllocFailed => f.write_str("Remote TTY allocation failed"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}

fn to_owned_verbose_error(mut v: VerboseError<&[u8]>) -> VerboseError<RawBytes<Vec<u8>>> {
    VerboseError {
        errors: v
            .errors
            .drain(..)
            .map(|(i, k)| (RawBytes(i.to_vec()), k))
            .collect(),
    }
}

impl From<nom::Err<VerboseError<&[u8]>>> for Error {
    fn from(e: nom::Err<VerboseError<&[u8]>>) -> Self {
        match e {
            nom::Err::Error(v) | nom::Err::Failure(v) => Self::Parsing(to_owned_verbose_error(v)),
            nom::Err::Incomplete(n) => Self::Incomplete(n),
        }
    }
}
