use indoc::formatdoc;
use std::ffi::{OsStr, OsString};
use std::fs::{create_dir_all, write};
use std::io;
use std::path::Path;
use std::process::Command;

use log::{debug, info};
use tempfile::TempDir;

use crate::Result;

trait ExtCommand {
    /// Join arguments for logging purpose
    fn join_args<Sep: AsRef<OsStr>>(&self, sep: Sep) -> OsString;
    /// Execute a command and check its result
    fn checked_run(&mut self) -> io::Result<()>;
}

impl ExtCommand for Command {
    fn join_args<Sep: AsRef<OsStr>>(&self, sep: Sep) -> OsString {
        match self.get_args().count() {
            0 => self.get_program().to_owned(),
            _ => {
                let cap = self.get_args().fold(self.get_program().len(), |acc, arg| {
                    acc + sep.as_ref().len() + arg.len()
                });

                let mut ret = OsString::with_capacity(cap);
                ret.push(self.get_program());
                self.get_args().for_each(|arg| {
                    ret.push(sep.as_ref());
                    ret.push(arg);
                });
                ret
            }
        }
    }

    fn checked_run(&mut self) -> io::Result<()> {
        debug!("Running command: {}", self.join_args(" ").to_string_lossy());

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

fn ostree() -> Command {
    Command::new("ostree")
}

/// Ensure publishing repo is there
pub fn ensure_repo<P: AsRef<Path>>(repo_dir: P) -> Result<()> {
    if repo_dir.as_ref().exists() {
        // assuming all good
        return Ok(());
    }

    ostree()
        .arg("init")
        .arg("--repo")
        .arg(repo_dir.as_ref().as_os_str())
        .arg("--mode=archive-z2")
        .checked_run()?;

    Ok(())
}

/// Run build process
pub fn run<RepoP, LayerP, I>(
    repo_dir: RepoP,
    layers: I,
    id: &str,
    arch: &str,
    ver: &str,
) -> Result<()>
where
    RepoP: AsRef<Path>,
    LayerP: AsRef<Path>,
    I: IntoIterator<Item = LayerP>,
{
    let tmpdir = TempDir::new()?;
    let tmp_repo = Path::join(tmpdir.path(), "repo");

    info!("Initializing temporary repo");
    ostree()
        .args(["init", "--mode=bare-user-only", "--repo"])
        .arg(tmp_repo.as_os_str())
        .current_dir(tmpdir.path())
        .checked_run()?;

    let base_branch = format!("base/{id}/{arch}/{ver}");
    let runtime_branch = format!("runtime/{id}/{arch}/{ver}");

    info!("Commiting initial build");
    ostree()
        .arg("commit")
        .arg("--repo")
        .arg(tmp_repo.as_os_str())
        .args(["-b", base_branch.as_str()])
        .args(layers.into_iter().map(|l| {
            let mut r = OsString::from("--tree=tar=");
            r.push(l.as_ref().as_os_str());
            r
        }))
        .current_dir(tmpdir.path())
        .checked_run()?;

    let subtree = Path::join(tmpdir.path(), "subtree");
    create_dir_all(subtree.as_path())?;

    info!("Commiting subtree");
    ostree()
        .arg("checkout")
        .arg("--repo")
        .arg(tmp_repo.as_os_str())
        .args(["--subpath", "/usr"])
        .arg("-U")
        .arg(base_branch.as_str())
        .arg(Path::join(subtree.as_path(), "files").as_os_str())
        .current_dir(tmpdir.path())
        .checked_run()?;

    // create metadata
    let metadata = formatdoc!(
        "\
	[Runtime]
        name={id}
        arch={arch}
        version={ver}"
    );

    write(
        Path::join(subtree.as_path(), "metadata").as_os_str(),
        metadata.as_str(),
    )?;

    ostree()
        .arg("commit")
        .arg("--repo")
        .arg(tmp_repo.as_os_str())
        .args([
            "--no-xattrs",
            "--owner-uid=0",
            "--owner-gid=0",
            "--link-checkout-speedup",
        ])
        .args(["-s", "Commit"])
        .args(["--branch", runtime_branch.as_str()])
        .arg(subtree.as_os_str())
        .args([
            "--add-metadata-string",
            format!("xa.metadata={}", metadata).as_str(),
        ])
        .current_dir(tmpdir.path())
        .checked_run()?;

    info!("Publishing");
    ostree()
        .arg("pull-local")
        .arg("--repo")
        .arg(repo_dir.as_ref().as_os_str())
        .arg(tmp_repo.as_os_str())
        .arg(runtime_branch.as_str())
        .current_dir(tmpdir.path())
        .checked_run()?;

    Command::new("flatpak")
        .arg("build-update-repo")
        .arg(repo_dir.as_ref().as_os_str())
        .current_dir(tmpdir.path())
        .checked_run()?;

    Ok(())
}
