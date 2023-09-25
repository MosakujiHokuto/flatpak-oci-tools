use clap;

use crate::Result;
use crate::obs;

#[derive(clap::Args)]
pub struct Args {
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

pub fn run(args: &Args) -> Result<()> {
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

    obsapi.download_binary(
        picked,
        args.dir.as_deref(),
        args.output.as_deref(),
    )?;

    Ok(())
}
