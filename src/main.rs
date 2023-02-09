use std::env;

use ssh_control::{Result, SshControl};

fn main() -> Result<()> {
    env_logger::init();

    let mut ctrl = SshControl::new(env::args().nth(1).unwrap())?;
    ctrl.check_alive()?;
    ctrl.new_session("id")?;
    let msg = ctrl.recv()?;
    dbg!(msg);

    Ok(())
}
