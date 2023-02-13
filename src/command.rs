use std::{collections::HashMap, fmt, fs::File};

mod pipe;
pub use pipe::{Pipe, PipeRead, PipeWrite};

pub struct Child {
    pub stdin: Option<PipeWrite>,
    pub stdout: Option<PipeRead>,
    pub stderr: Option<PipeRead>,
    pub(super) session: u32,
    pub(super) _devnull: Option<File>,
}

impl fmt::Debug for Child {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Child")
            .field("stdin", &self.stdin)
            .field("stdout", &self.stdout)
            .field("stderr", &self.stderr)
            .field("session", &self.session)
            .finish_non_exhaustive()
    }
}

/// SSH Command struct, mimicks [`std::process::Command`] interface
#[derive(Debug)]
pub struct SshCommand {
    pub(super) shell_command: String,
    pub(super) environment: HashMap<String, String>,
    pub(super) want_tty: bool,
    pub(super) want_x11_forwarding: bool,
    pub(super) stdin: Option<Pipe>,
    pub(super) stdout: Option<Pipe>,
    pub(super) stderr: Option<Pipe>,
}

impl SshCommand {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            shell_command: command.into(),
            environment: HashMap::default(),
            want_tty: false,
            want_x11_forwarding: false,
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }

    pub fn env(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    pub fn copy_env(&mut self) -> &mut Self {
        for (key, value) in std::env::vars() {
            self.env(key, value);
        }
        self
    }

    pub fn env_remove(&mut self, key: impl AsRef<str>) -> &mut Self {
        self.environment.remove(key.as_ref());
        self
    }

    pub fn stdin(&mut self, p: Pipe) -> &mut Self {
        self.stdin = Some(p);
        self
    }

    pub fn stdout(&mut self, p: Pipe) -> &mut Self {
        self.stdout = Some(p);
        self
    }

    pub fn stderr(&mut self, p: Pipe) -> &mut Self {
        self.stderr = Some(p);
        self
    }
}
