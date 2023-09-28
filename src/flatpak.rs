use indoc::formatdoc;
use std::ffi::OsString;
use std::fs::{create_dir_all, write, read_to_string};
use std::path::Path;
use std::process::Command;

use log::info;
use tempfile::TempDir;

use crate::Result;
use crate::exec::{CheckedRun, ostree};

/// Ensure publishing repo is there
pub fn ensure_repo<P: AsRef<Path>>(repo_dir: P) -> Result<()> {
    if repo_dir.as_ref().exists() {
        // assuming all good
        return Ok(());
    }

    create_dir_all(repo_dir.as_ref())?;

    ostree()
        .arg("init")
        .arg("--repo")
        .arg(repo_dir.as_ref().as_os_str())
        .arg("--mode=archive-z2")
        .checked_run()?;

    Ok(())
}

pub struct Builder {
    tmpdir: TempDir,
}

/// Run build process
impl Builder {
    pub fn new() -> Result<Builder> {
        let tmpdir = TempDir::new_in("/var/tmp")?;
        let tmp_repo = Path::join(tmpdir.path(), "repo");

        info!("Initializing temporary repo");
        ostree()
            .args(["init", "--mode=bare-user-only", "--repo"])
            .arg(tmp_repo.as_os_str())
            .current_dir(tmpdir.path())
            .checked_run()?;

        Ok(Builder { tmpdir })
    }

    pub fn build_runtime<RepoP, LayerP, I>(
        &self,
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
        let tmp_repo = Path::join(self.tmpdir.path(), "repo");

        let base_branch = format!("base");
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
            .current_dir(self.tmpdir.path())
            .checked_run()?;

        let subtree = Path::join(self.tmpdir.path(), "subtree");
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
            .current_dir(self.tmpdir.path())
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
            .current_dir(self.tmpdir.path())
            .checked_run()?;

        info!("Publishing");
        ostree()
            .arg("pull-local")
            .arg("--repo")
            .arg(repo_dir.as_ref().as_os_str())
            .arg(tmp_repo.as_os_str())
            .arg(runtime_branch.as_str())
            .current_dir(self.tmpdir.path())
            .checked_run()?;

        Command::new("flatpak")
            .arg("build-update-repo")
            .arg(repo_dir.as_ref().as_os_str())
            .current_dir(self.tmpdir.path())
            .checked_run()?;

        Ok(())
    }

    pub fn build_app<P: AsRef<Path>>(
        &self,
        repo_dir: P,
        id: &str,
	runtime: &str,
        _arch: &str,
        ver: &str,
    ) -> Result<()> {
        let tmp_repo = Path::join(self.tmpdir.path(), "repo");

        let base_branch = format!("base");

	let manifest = format!("{id}.yaml");

	let build_dir = self.tmpdir.path().join("app");

	println!("Checking out flatpak.yaml");
	ostree()
	    .arg("checkout")
	    .arg("--repo")
	    .arg(tmp_repo.as_os_str())
	    .arg("--subpath=/flatpak.yaml")
	    .arg("-U")
	    .arg(base_branch.as_str())
	    .arg(build_dir.as_os_str())
	    .checked_run()?;

	println!("Generating manifest");
	let content = read_to_string(build_dir.join("flatpak.yaml"))?;

	// XXX we should probably optimize this
	let content = content.replace("%FLATPAK_OCI_APPID%", id);
	let content = content.replace("%FLATPAK_OCI_RUNTIMEID%", runtime);
	let content = content.replace("%FLATPAK_OCI_RUNTIMEVER%", ver);

	write(build_dir.join(&manifest), content)?;

	println!("Building application");
	Command::new("flatpak-builder")
	    .arg("--repo")
	    .arg(repo_dir.as_ref().as_os_str())
	    .arg("build")
	    .arg(&manifest)
	    .current_dir(build_dir.as_path())
	    .checked_run()?;

	Ok(())
    }
}
