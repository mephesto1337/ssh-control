use passfd::FdPassingExt;
use std::{
    env, fs,
    mem::ManuallyDrop,
    os::unix::{io::AsRawFd, net::UnixStream},
    path::Path,
};

mod protocol;
pub use protocol::{
    client::{self, MuxMessage},
    server::{self, MuxResponse},
    Hello, Packet, Wire,
};

pub mod command;
use command::{Child, SshCommand};

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

    fn recv_helper(&mut self) -> Result<MuxResponse> {
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

    fn recv<'a, T>(&'a mut self) -> Result<T>
    where
        MuxResponse<'a>: Into<Result<T>>,
    {
        let msg = self.recv_helper()?;
        msg.into()
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
        let alive: server::Alive = self.recv()?;
        Ok(alive.server_pid)
    }

    pub fn new_session(&mut self, command: SshCommand) -> Result<Child> {
        let environment: Vec<_> = command
            .environment
            .iter()
            .map(|(k, v)| format!("{k}={v}").into())
            .collect();
        let req: MuxMessage = client::NewSession {
            request_id: 0,
            want_tty: command.want_tty,
            want_x11_forwarding: command.want_x11_forwarding,
            want_agent: false,
            subsystem: false,
            escape_char: b'~' as u32,
            terminal_type: env::var("TERM")
                .ok()
                .unwrap_or_else(|| "xterm".into())
                .into(),
            command: command.shell_command.into(),
            environment,
        }
        .into();
        self.send(req)?;

        let devnull =
            if command.stdin.is_some() || command.stdout.is_some() || command.stderr.is_some() {
                Some(
                    fs::OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open("/dev/null")?,
                )
            } else {
                None
            };

        let (child_stdin, stdin) = match command.stdin {
            Some(p) => (Some(p.write), p.read),
            None => (
                None,
                command::PipeRead(
                    devnull
                        .as_ref()
                        .map(|f| f.as_raw_fd())
                        .unwrap_or_else(|| std::io::stdin().lock().as_raw_fd()),
                ),
            ),
        };
        let (child_stdout, stdout) = match command.stdout {
            Some(p) => (Some(p.read), p.write),
            None => (
                None,
                command::PipeWrite(
                    devnull
                        .as_ref()
                        .map(|f| f.as_raw_fd())
                        .unwrap_or_else(|| std::io::stdout().lock().as_raw_fd()),
                ),
            ),
        };
        let (child_stderr, stderr) = match command.stderr {
            Some(p) => (Some(p.read), p.write),
            None => (
                None,
                command::PipeWrite(
                    devnull
                        .as_ref()
                        .map(|f| f.as_raw_fd())
                        .unwrap_or_else(|| std::io::stderr().lock().as_raw_fd()),
                ),
            ),
        };
        self.socket.send_fd_with_payload(stdin.as_raw_fd(), 0u8)?;
        self.socket.send_fd_with_payload(stdout.as_raw_fd(), 0u8)?;
        self.socket.send_fd_with_payload(stderr.as_raw_fd(), 0u8)?;

        let so: server::SessionOpened = self.recv()?;
        Ok(Child {
            stdin: child_stdin,
            stdout: child_stdout,
            stderr: child_stderr,
            session: so.session_id,
            _devnull: devnull,
        })
    }

    pub fn wait(&mut self, child: &Child) -> Result<bool> {
        let server::ExitMessage { session_id, .. } = self.recv()?;
        if session_id != child.session {
            log::warn!(
                "Session {session_id} was joined, but {} was expected",
                child.session
            );
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn new_stdio_forward(
        &mut self,
        host: impl AsRef<str>,
        port: client::Port,
        pipe: Option<command::Pipe>,
    ) -> Result<u32> {
        let req: MuxMessage = client::NewStdioFwd {
            request_id: 0,
            connect_host: host.as_ref().into(),
            connect_port: port,
        }
        .into();
        self.send(req)?;
        let pipe = pipe.unwrap_or_else(command::Pipe::stdio);
        self.socket.send_fd_with_payload(pipe.read.0, 0u8)?;
        self.socket.send_fd_with_payload(pipe.write.0, 0u8)?;

        // Avoid pipe being closed
        let _ = ManuallyDrop::new(pipe);

        let so: server::SessionOpened = self.recv()?;

        Ok(so.session_id)
    }
}
