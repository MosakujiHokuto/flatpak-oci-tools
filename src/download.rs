use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::blocking::Response;
use reqwest::header::CONTENT_LENGTH;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;

use crate::Result;

pub fn run<P: AsRef<Path>>(dst: P, mut res: Response) -> Result<()> {
    let len = res
        .headers()
        .get(CONTENT_LENGTH)
        .ok_or("Response doesn't have content length")?;
    let len = u64::from_str(len.to_str()?).map_err(|_| "invalid content length")?;

    let dst_name = dst.as_ref().file_name().unwrap().to_string_lossy().to_string();
    let mut dst = File::create(&dst).or(Err(format!(
        "Unable to create {}",
        dst.as_ref().display()
    )))?;
    let mut bytes_read: u64 = 0;

    let mut buf = vec![0; 10240];

    // Initialize progress
    let pb = ProgressBar::new(len);
    pb.set_style(ProgressStyle::with_template(
	"{spinner:.green} [{elapsed_precise}] {msg:20!} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} (ETA {eta})")?
	    .with_key("eta", |state: &ProgressState, w: &mut dyn fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()));
    pb.set_message(dst_name);

    loop {
        let ret = res.read(&mut buf)?;
        if ret == 0 {
            break;
        }
        dst.write_all(&buf[..ret])?;
        let ret: u64 = ret.try_into()?;
        bytes_read += ret;
        pb.set_position(bytes_read);
    }

    pb.finish();

    Ok(())
}
