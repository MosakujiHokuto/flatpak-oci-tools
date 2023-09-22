use log::debug;
use reqwest::header::CONTENT_LENGTH;
use serde::Deserialize;
use std::fs::File;
use std::fmt::Debug;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use super::ObsApi;

// OBS API structures
#[derive(Deserialize)]
struct ObsBinaryList {
    binary: Vec<ObsBinary>,
}

#[derive(Deserialize)]
struct ObsBinary {
    #[serde(rename = "@filename")]
    filename: String,
    #[serde(rename = "@size")]
    size: u64,
    #[serde(rename = "@mtime")]
    mtime: u64,
}

// Binary file handler
#[derive(Debug)]
pub struct Binary {
    pub filename: String,
    pub size: u64,
    pub mtime: u64,

    pub project: String,
    pub repository: String,
    pub architecture: String,
    pub package: String,
}

impl ObsApi {
    pub fn list_binaries(&self, proj: &str, repo: &str, arch: &str, pkg: &str) ->
	Result<Vec<Binary>, Box<dyn std::error::Error>>
    {
	let path = format!("build/{proj}/{repo}/{arch}/{pkg}");
	let res: ObsBinaryList = self.get(&path)?;
	let ret = res.binary
	    .into_iter()
	    .map(|i| Binary {
		filename: i.filename,
		size: i.size,
		mtime: i.mtime,

		project: proj.to_string(),
		repository: repo.to_string(),
		architecture: arch.to_string(),
		package: pkg.to_string()
	    })
	    .collect();
	Ok(ret)
    }

    pub fn download_binary<P: AsRef<Path>>(
	&self, bin: &Binary, dir_path: Option<P>,
	output_name: Option<&str>,
	mut prog_cb: Box<dyn FnMut(usize, usize)>) ->
	Result<(), Box<dyn std::error::Error>>
    where
    {
	let req_path = format!("build/{proj}/{repo}/{arch}/{pkg}/{file}",
			  proj = bin.project,
			  repo = bin.repository,
			  arch = bin.architecture,
			  pkg = bin.package,
			  file = bin.filename
	);
	let mut res = self
	    .do_get_request(&req_path)
	    .send()?;
	let len = res
	    .headers()
	    .get(CONTENT_LENGTH)
	    .ok_or("Response doesn't have content length")?;
	let len = usize::from_str(len.to_str()?)
	    .map_err(|_| "invalid content length")?;

	let output = output_name.unwrap_or(&bin.filename);
	let file_path = dir_path
	    .and_then(|p| Some(p.as_ref().join(output)))
	    .unwrap_or_else(|| PathBuf::from(output));

	let mut file = File::create(&file_path)
	    .or(Err(format!("Unable to create {}", file_path.display())))?;
	let mut bytes_read: usize = 0;

	let mut buf = vec![0; 10240];

	// Initialize progress
	prog_cb(0, len);

	loop {
	    let ret = res.read(&mut buf)?;
	    debug!("Incoming: {} bytes", ret);
	    if ret == 0 {
		break;
	    }
	    bytes_read += ret;
	    file.write_all(&buf[..ret])?;
	    prog_cb(bytes_read, len);
	}

	// Finish progress report
	prog_cb(0, 0);

	Ok(())
    }
}
