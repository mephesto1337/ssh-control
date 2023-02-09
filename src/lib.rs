use std::{os::unix::net::UnixStream, path::Path};

mod protocol;
use protocol::{
    client,
    server::{self, MuxResponse},
    Hello, MuxMessage, Packet, Wire,
};

mod error;
pub use error::{Error, Result};

pub struct SshControl {
    socket: UnixStream,
    buffer: Packet,
    request_id: u32,
}

impl SshControl {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let socket = UnixStream::connect(path)?;
        let buffer = Vec::with_capacity(1024).into();

        let mut me = Self {
            socket,
            buffer,
            request_id: 1,
        };
        me.send_hello()?;

        Ok(me)
    }

    fn send<T: Wire>(&mut self, obj: &T) -> Result<()> {
        self.buffer.set(obj);
        self.buffer.serialize(&mut self.socket)?;
        Ok(())
    }

    fn recv<T>(&mut self) -> Result<T>
    where
        T: Wire,
    {
        self.buffer.recv_next(&mut self.socket)
    }

    fn send_hello(&mut self) -> Result<()> {
        let hello = Hello {
            version: 4,
            extensions: Vec::new(),
        };
        self.send(&hello)?;

        let hello = self.recv::<Hello>()?;
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

    pub fn check_alive(&mut self) -> Result<bool> {
        let request_id = self.request_id.wrapping_add(1);
        let check = MuxMessage::AliveCheck(client::AliveCheck { request_id });
        self.send(&check)?;
        let response = self.recv::<MuxResponse>()?;
        Ok(match response {
            MuxResponse::Alive(a) => {
                log::info!("Server pid: {}", a.server_pid);
                true
            }
            _ => false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        env_logger::init();

        let mut ctrl =
            SshControl::new("/home/thomas/tmp/ssh-thomas@pentest-arch.vms.local:22.sock").unwrap();
        ctrl.check_alive().unwrap();
    }
}
