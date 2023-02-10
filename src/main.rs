use std::{env, fs, io};

use ssh_control::{
    command::{Pipe, SshCommand},
    Result, SshControl,
};

fn main() {
    env_logger::builder()
        .parse_default_env()
        .target(env_logger::Target::Stderr)
        .init();

    if let Err(e) = main_helper() {
        log::error!("{e}");
    }
}

fn main_helper() -> Result<()> {
    let mut ctrl = SshControl::new(env::args().nth(1).unwrap())?;
    let server_pid = ctrl.check_alive()?;
    log::info!("Server pid: {server_pid}");

    let mut cmd = SshCommand::new("id");
    cmd.stderr(Pipe::dev_null()?);
    cmd.stdout(Pipe::new()?);

    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(false)
        .open("/tmp/test")?;

    let mut child = ctrl.new_session(cmd)?;

    let n = io::copy(child.stdout.as_mut().unwrap(), &mut f)?;
    log::info!("Copy {n} bytes from command to file");

    ctrl.wait(&child)?;

    Ok(())
}
