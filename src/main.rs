use clap::{Args, Parser, Subcommand};
use log::debug;
use std::error::Error;
use std::process::Command;
use std::io;

mod oci;
mod workdir;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    ImportContainer(ImportContainerArgs),
}

#[derive(Args)]
struct ImportContainerArgs {
    image_file: String
}

fn check_run(argv: &[&str]) -> io::Result<()> {
    debug!("[check_run] Runing command: {}", argv.join(" "));
    let mut cmd = Command::new(argv[0]);
    for a in &argv[1..] {
        cmd.arg(a);
    }

    let mut child = cmd.spawn().or_else(|err| {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to spawn command {}: {}", argv[0], err)))
    })?;

    let ecode = child.wait()?;
    if !ecode.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Command {} exited with status {}", argv[0], ecode)));
    }

    Ok(())
}

fn import_container(args: &ImportContainerArgs) -> Result<(), Box<dyn Error>> {
    println!("Importing {}", args.image_file);
    let mut img = oci::Image::new(args.image_file.as_str())?;

    let work_dir = workdir::WorkDir::new()?;

    let build_dir = work_dir.subdir("build")?;
    let image_dir = work_dir.subdir("image")?;

    img.unpack_fs_layers(build_dir.path(), image_dir.path())?;

    let _pushd = build_dir.pushd();
    check_run(&["ls"])?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
	Commands::ImportContainer(args) => import_container(args)
    }
}
