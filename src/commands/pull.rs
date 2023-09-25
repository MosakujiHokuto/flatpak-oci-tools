use clap;

use crate::Result;
use crate::oci;

#[derive(clap::Args)]
pub struct Args {
    #[arg(long, default_value = "https://registry.opensuse.org")]
    registry: String,
    #[arg(long, default_value = "home:yudaike:flatpak-oci-container")]
    project: String,
    #[arg(long, default_value = "images")]
    repo: String,

    container: String,
}

pub fn run(args: &Args) -> Result<()> {
    let (container, tag) = args
        .container
        .rsplit_once(":")
        .unwrap_or((&args.container, "latest"));

    let container_name = format!(
        "{proj}/{repo}/{container}",
        proj = args.project.replace(":", "/"),
        repo = args.repo
    );

    let api = oci::Api::new(&args.registry)?;
    let manifest = api.get_manifest(&container_name, tag)?;

    println!("{:?}", manifest);

    Ok(())
}
