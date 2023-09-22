use clap::{Args, Parser, Subcommand};
use indoc::formatdoc;
use indicatif::ProgressBar;
use log::{debug, info};
use std::error::Error;
use std::fs;
use std::hash::Hasher;
use std::io;
use std::os::unix::fs::symlink;
use std::process::Command;
use tempfile::TempDir;

mod obs;
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
    Fetch(FetchArgs),
}

#[derive(Args)]
struct ImportContainerArgs {
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

#[derive(Args)]
struct FetchArgs {
    #[arg(short, long, default_value = "https://api.opensuse.org")]
    api: String,
    #[arg(short, long)]
    username: String,
    #[arg(short, long)]
    password: String,
    #[arg(short, long)]
    dir: Option<String>,
    #[arg(short, long)]
    output: Option<String>,

    project: String,
    obs_repositoroy: String,
    arch: String,
    package: String,
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
            format!("Failed to spawn command {}: {}", argv[0], err),
        ))
    })?;

    let ecode = child.wait()?;
    if !ecode.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Command {} exited with status {}", argv[0], ecode),
        ));
    }

    Ok(())
}

fn import_container(args: &ImportContainerArgs) -> Result<(), Box<dyn Error>> {
    println!("Importing {}", args.image_file);
    let mut img = oci::Image::new(args.image_file.as_str())?;

    let work_dir = workdir::WorkDir::new()?;

    let build_dir = work_dir.subdir("build")?;
    let image_dir = work_dir.subdir("image")?;
    let repo_dir = work_dir.subdir("repo")?;

    let id = args.id.as_str();
    let arch = args.arch.as_str();
    let version = args.version.as_str();

    info!("Unpacking image");
    img.unpack_fs_layers(build_dir.path(), image_dir.path())?;

    info!("Initializing OSTree repo");
    check_run(&[
        "ostree",
        "init",
        "--mode=bare-user-only",
        "--repo",
        repo_dir.path_str().unwrap(),
    ])?;

    {
        let _pushd = build_dir.pushd();

        // prepare usr
        info!("Preparing build root");
        fs::create_dir_all("usr/share/fonts")?;
        symlink("/run/host/fonts", "usr/share/fonts/flatpakhostfonts")?;
        check_run(&["cp", "-r", "etc", "usr/etc"])?;
    }

    let base_branch = format!(
        "base/{id}/{arch}/{version}",
        id = id,
        arch = arch,
        version = version
    );
    let runtime_branch = format!(
        "runtime/{id}/{arch}/{version}",
        id = id,
        arch = arch,
        version = version
    );

    info!("Commiting initial build");
    check_run(&[
        "ostree",
        "commit",
        "--repo",
        repo_dir.path().as_os_str().to_str().unwrap(),
        "-b",
        base_branch.as_str(),
        format!("--tree=dir={}", build_dir.path().display()).as_str(),
    ])?;
    {
        info!("Commiting subtree");
        let subtree_dir = TempDir::new_in(work_dir.path())?;

        check_run(&[
            "ostree",
            "checkout",
            "--repo",
            repo_dir.path_str().unwrap(),
            "--subpath",
            "/usr",
            "-U",
            base_branch.as_str(),
            subtree_dir
                .path()
                .join("files")
                .as_os_str()
                .to_str()
                .unwrap(),
        ])?;

        let metadata = formatdoc!(
            "\
	    [Runtime]
            name={name}
            arch={arch}
            version={version}",
            name = id,
            arch = arch,
            version = version
        );

        fs::write(
            subtree_dir
                .path()
                .join("metadata")
                .as_os_str()
                .to_str()
                .unwrap(),
            metadata.as_str(),
        )?;

        check_run(&[
            "ostree",
            "commit",
            "--repo",
            repo_dir.path_str().unwrap(),
            "--no-xattrs",
            "--owner-uid=0",
            "--owner-gid=0",
            "--link-checkout-speedup",
            "-s",
            "Commit",
            "--branch",
            runtime_branch.as_str(),
            subtree_dir.path().to_str().unwrap(),
            "--add-metadata-string",
            format!("xa.metadata={}", metadata).as_str(),
        ])?;
    }

    info!("Pulling into specified repo");
    check_run(&[
        "ostree",
        "pull-local",
        "--repo",
        args.repo.as_str(),
        repo_dir.path_str().unwrap(),
        runtime_branch.as_str(),
    ])?;
    check_run(&["flatpak", "build-update-repo", "/tmp/ocirepo"])?;

    Ok(())
}

fn fetch(args: &FetchArgs) -> Result<(), Box<dyn Error>> {
    let obsapi = obs::ObsApi::new(
        args.api.as_str(),
        args.username.as_str(),
        args.password.as_str(),
    )?;

    let res = obsapi.list_binaries(
        &args.project,
        &args.obs_repositoroy,
        &args.arch,
        &args.package,
    )?;

    let candidates: Vec<&obs::Binary> = res
        .iter()
        .filter(|i| i.filename.ends_with(".docker.tar"))
        .collect();

    if candidates.len() != 1 {
	if candidates.len() == 0 {
	    println!("No candidates available");
	} else {
	    println!("Multiple candidates detected");
	}
	std::process::exit(-1);
    }

    let picked = candidates.first().unwrap();
    println!("Picked file: {}", picked.filename);

    let mut pb = None;
    obsapi.download_binary(
	picked, args.dir.as_deref(), args.output.as_deref(),
	Box::new(move |read, total| {
	    if pb.is_none() {
		pb = Some(ProgressBar::new(total.try_into().unwrap()));
		return
	    }

	    let pb = pb.as_ref().unwrap();

	    if read == 0 && total == 0 {
		pb.finish_with_message("Download finished successfully");
		return
	    }

	    pb.set_position(std::cmp::min(read, total).try_into().unwrap());
	}))?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::ImportContainer(args) => import_container(args),
        Commands::Fetch(args) => fetch(args),
    }
}
