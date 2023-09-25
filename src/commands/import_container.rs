use clap;
use std::fs::create_dir_all;
use std::path::Path;
use tempfile::TempDir;

use crate::flatpak;
use crate::obs;
use crate::Result;

#[derive(clap::Args)]
pub struct Args {
    // XXX Those defaults should be derived from image
    #[arg(long, default_value = "org.openSUSE.OCIPlatform")]
    id: String,
    #[arg(long, default_value = "x86_64")]
    arch: String,
    #[arg(long, default_value = "1")]
    version: String,

    image_file: String,
    repo: String,
}

pub fn run(args: &Args) -> Result<()> {
    println!("Importing {}", args.image_file);

    let id = args.id.as_str();
    let arch = args.arch.as_str();
    let version = args.version.as_str();

    let work_dir = TempDir::new()?;
    let work_dir_path = work_dir.path();

    // Unpacking image
    let mut img = obs::ContainerImage::new(args.image_file.as_str())?;
    let image_dir = Path::join(work_dir_path, "image");
    create_dir_all(image_dir.as_path())?;

    println!("Unpacking image");
    img.unpack(image_dir.as_path())?;

    flatpak::Builder::new()?.build_runtime(
        &args.repo,
        img.layers()
            .unwrap()
            .iter()
            .map(|l| Path::join(image_dir.as_path(), l)),
        id,
        arch,
        version,
    )?;

    Ok(())
}
