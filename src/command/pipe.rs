use std::{
    fs,
    io::{self, Read, Write},
    mem::MaybeUninit,
    os::{
        fd::IntoRawFd,
        unix::io::{AsRawFd, RawFd},
    },
};

#[derive(Debug)]
pub struct PipeRead(pub(crate) RawFd);

#[derive(Debug)]
pub struct PipeWrite(pub(crate) RawFd);

#[derive(Debug)]
pub struct Pipe {
    pub(crate) read: PipeRead,
    pub(crate) write: PipeWrite,
}

impl Pipe {
    pub fn new() -> io::Result<Self> {
        let mut fds: MaybeUninit<[RawFd; 2]> = MaybeUninit::uninit();
        let ret = unsafe { libc::pipe(fds.as_mut_ptr().cast()) };
        if ret == 0 {
            let fds = unsafe { fds.assume_init() };
            Ok(Self {
                read: PipeRead(fds[0]),
                write: PipeWrite(fds[1]),
            })
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub fn dev_null() -> io::Result<Self> {
        let r = fs::OpenOptions::new().read(true).open("/dev/null")?;
        let w = fs::OpenOptions::new().write(true).open("/dev/null")?;
        let read = PipeRead(r.into_raw_fd());
        let write = PipeWrite(w.into_raw_fd());
        Ok(Self { read, write })
    }

    pub fn with_pipes(read: PipeRead, write: PipeWrite) -> Self {
        Self { read, write }
    }

    pub fn stdio() -> Self {
        let stdin = io::stdin().lock().as_raw_fd();
        let stdout = io::stdout().lock().as_raw_fd();
        Self {
            read: PipeRead(stdin),
            write: PipeWrite(stdout),
        }
    }
}

impl AsRawFd for PipeRead {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl AsRawFd for PipeWrite {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl Read for PipeRead {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe { libc::read(self.0, buf.as_mut_ptr().cast(), buf.len()) };
        if ret >= 0 {
            Ok(ret as usize)
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

impl Write for PipeWrite {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let ret = unsafe { libc::write(self.0, buf.as_ptr().cast(), buf.len()) };
        if ret >= 0 {
            Ok(ret as usize)
        } else {
            Err(io::Error::last_os_error())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        // There is no buffering
        Ok(())
    }
}

impl Drop for PipeRead {
    fn drop(&mut self) {
        let ret = unsafe { libc::close(self.0) };
        if ret < 0 {
            log::warn!("Could not close PipeRead {}", self.0);
        }
    }
}

impl Drop for PipeWrite {
    fn drop(&mut self) {
        let ret = unsafe { libc::close(self.0) };
        if ret < 0 {
            log::warn!("Could not close PipeWrite {}", self.0);
        }
    }
}

impl Read for Pipe {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read.read(buf)
    }
}

impl Write for Pipe {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.write.flush()
    }
}
