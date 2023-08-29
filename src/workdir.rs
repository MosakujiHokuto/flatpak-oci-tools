use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use log::debug;
use tempfile::TempDir;

pub struct WorkDir {
    work_dir: TempDir,
}

pub struct SubDir {
    path: PathBuf,
}

pub struct PushD {
    prev_path: PathBuf,
}

impl WorkDir {
    pub fn new() -> io::Result<WorkDir> {
        let work_dir = TempDir::with_prefix("flatpak-oci-tools-")?;

        Ok(WorkDir { work_dir })
    }

    pub fn path(&self) -> &Path {
	self.work_dir.path()
    }

    pub fn subdir<P: AsRef<Path>>(&self, path: P) -> io::Result<SubDir> {
        let path = self.work_dir.path().join(path);

        fs::create_dir(path.as_path())?;

        Ok(SubDir { path })
    }
}

impl SubDir {
    /**
     * Create a sub directory under current working dir
     */
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<SubDir> {
	let path = env::current_dir()?.join(path);

	fs::create_dir(path.as_path())?;

	Ok(SubDir { path })
    }

    pub fn cd(&self) -> io::Result<()> {
        debug!("[SubDir] Entering {}", self.path.display());
        env::set_current_dir(self.path.as_path())
    }

    pub fn path(&self) -> &Path {
	self.path.as_path()
    }

    pub fn path_str(&self) -> Option<&str> {
	self.path.to_str()
    }

    pub fn pushd(&self) -> io::Result<PushD> {
        let prev_path = env::current_dir()?;
        self.cd()?;
        Ok(PushD { prev_path })
    }
}

pub fn pushd<P: AsRef<Path>>(path: P) -> io::Result<PushD> {
    let prev_path = env::current_dir()?;
    debug!("[PushD] Entering {}", path.as_ref().display());
    env::set_current_dir(path)?;
    Ok(PushD { prev_path })
}

impl Drop for PushD {
    fn drop(&mut self) {
        debug!("[PushD] Entering {}", self.prev_path.display());
        env::set_current_dir(self.prev_path.as_path()).unwrap();
    }
}
