use passfd::FdPassingExt;
use std::{
    os::unix::{io::AsRawFd, net::UnixStream},
    path::Path,
};

mod protocol;
pub use protocol::{
    client::{self, MuxMessage},
    server::{self, MuxResponse},
    Hello, Packet, Wire,
};

pub(crate) mod error;
pub use error::{Error, Result};

pub struct SshControl {
    socket: UnixStream,
    buffer: Packet,
    request_id: u32,
    expected_request_id: Option<u32>,
}

impl SshControl {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let socket = UnixStream::connect(path)?;
        let buffer = Vec::with_capacity(1024).into();

        let mut me = Self {
            socket,
            buffer,
            request_id: 0,
            expected_request_id: None,
        };
        me.send_hello()?;

        Ok(me)
    }

    fn get_next_request_id(&mut self) -> u32 {
        self.request_id.wrapping_add(1)
    }

    fn send<'a, T>(&mut self, obj: T) -> Result<()>
    where
        T: Into<MuxMessage<'a>>,
    {
        let mut msg = obj.into();
        let request_id = self.get_next_request_id();
        msg.set_request_id(request_id);
        self.expected_request_id = Some(request_id);
        self.buffer.set(&msg);
        self.buffer.serialize(&mut self.socket)?;
        Ok(())
    }

    pub fn recv(&mut self) -> Result<MuxResponse> {
        let response: MuxResponse = self.buffer.recv_next(&mut self.socket)?;
        let expected_request_id = self.expected_request_id.take();
        if response.get_request_id() != expected_request_id {
            log::error!("Request IDs does not match");
            Err(Error::InvalidResponseID {
                expected: expected_request_id,
                received: response.get_request_id(),
            })
        } else {
            Ok(response)
        }
    }

    fn send_hello(&mut self) -> Result<()> {
        let hello = Hello {
            version: 4,
            extensions: Vec::new(),
        };
        self.buffer.set(&hello);
        self.buffer.serialize(&mut self.socket)?;

        let hello = self.buffer.recv_next::<Hello, _>(&mut self.socket)?;
        log::debug!(
            "Server is running version {} with extensions: {:?}",
            hello.version,
            hello.extensions
        );
        if hello.version != 4 {
            return Err(Error::UnsupportedVersion(hello.version));
        }

        Ok(())
    }

    pub fn check_alive(&mut self) -> Result<u32> {
        let check: MuxMessage = client::AliveCheck { request_id: 0 }.into();
        self.send(check)?;
        let response = self.recv()?;
        match response {
            MuxResponse::Alive(a) => {
                log::info!("Server pid: {}", a.server_pid);
                Ok(a.server_pid)
            }
            _ => {
                log::error!("Alive check received bad response type: {response:?}");
                Err(Error::InvalidPacket {
                    description: format!("{response:#?}").into(),
                })
            }
        }
    }

    pub fn new_session(&mut self, command: impl AsRef<str>) -> Result<u32> {
        let req: MuxMessage = client::NewSession {
            request_id: 0,
            want_tty: false,
            want_x11_forwarding: false,
            want_agent: false,
            subsystem: false,
            escape_char: b'~' as u32,
            terminal_type: "tmux-256color".into(),
            command: command.as_ref().into(),
            environment: vec!["LANG=en_US.UTF-8".into(), "TERM=tmux-256color".into()],
        }
        .into();
        self.send(req)?;

        let stdin = std::io::stdin().lock();
        self.socket.send_fd(stdin.as_raw_fd())?;
        let stdout = std::io::stdout().lock();
        self.socket.send_fd(stdout.as_raw_fd())?;
        let stderr = std::io::stderr().lock();
        self.socket.send_fd(stderr.as_raw_fd())?;

        let response = self.recv()?;
        match response {
            MuxResponse::SessionOpened(so) => {
                log::info!("Session opened with ID: {}", so.session_id);
                Ok(so.session_id)
            }
            MuxResponse::PermissionDenied(pd) => {
                log::error!("Cannot open session (permission denied): {}", pd.reason);
                Ok(0)
            }
            MuxResponse::Failure(f) => {
                log::error!("Cannot open session (failure): {}", f.reason);
                Ok(0)
            }
            _ => {
                log::error!("NewSession check received bad response type: {response:?}");
                Err(Error::InvalidPacket {
                    description: format!("{response:#?}").into(),
                })
            }
        }
    }
}
