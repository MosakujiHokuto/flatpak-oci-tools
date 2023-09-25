use clap;
use indicatif::ProgressBar;

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

    let mut pb = None;
    obsapi.download_binary(
        picked,
        args.dir.as_deref(),
        args.output.as_deref(),
        Box::new(move |read, total| {
            if pb.is_none() {
                pb = Some(ProgressBar::new(total.try_into().unwrap()));
                return;
            }

            let pb = pb.as_ref().unwrap();

            if read == 0 && total == 0 {
                pb.finish_with_message("Download finished successfully");
                return;
            }

            pb.set_position(std::cmp::min(read, total).try_into().unwrap());
        }),
    )?;

    Ok(())
}
