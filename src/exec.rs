use log::debug;
use std::ffi::{OsStr, OsString};
use std::io;
use std::process::Command;

use crate::IS_USER;

pub fn ostree() -> Command {
    Command::new("ostree")
}

pub fn flatpak() -> Command {
    let mut cmd = Command::new("flatpak");
    if IS_USER.get().unwrap().to_owned() {
	cmd.arg("--user");
    }
    cmd
}

fn join_args<Sep: AsRef<OsStr>>(cmd: &Command, sep: Sep) -> OsString {
    match cmd.get_args().count() {
        0 => cmd.get_program().to_owned(),
        _ => {
            let cap = cmd.get_args().fold(cmd.get_program().len(), |acc, arg| {
                acc + sep.as_ref().len() + arg.len()
            });

            let mut ret = OsString::with_capacity(cap);
            ret.push(cmd.get_program());
            cmd.get_args().for_each(|arg| {
                ret.push(sep.as_ref());
                ret.push(arg);
            });
            ret
        }
    }
}

pub trait CheckedRun {
    fn checked_run(&mut self) -> io::Result<()>;
}

impl CheckedRun for Command {
    fn checked_run(&mut self) -> io::Result<()>
    {
        debug!("Running command: {}", join_args(self, " ").to_string_lossy());

        let mut child = self.spawn().or_else(|err| {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Failed to spawn command {}: {}",
                    self.get_program().to_string_lossy(),
                    err
                ),
            ))
        })?;

        let ecode = child.wait()?;
        if !ecode.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Command {} exited with status {}",
                    self.get_program().to_string_lossy(),
                    ecode
                ),
            ));
        }
        Ok(())
    }
}
