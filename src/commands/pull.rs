use clap;
use sha256::try_digest;
use std::fs::create_dir_all;
use std::path::PathBuf;

use crate::oci;
use crate::Result;

// XXX make this customizable
const FS_LAYERS_STORE: &str = "/tmp/var/lib/flatpak-oci-tools/layers";

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

/// Get cache location for a layer
fn get_layer_path(layer: &str) -> PathBuf {
    let mut ret = PathBuf::from(FS_LAYERS_STORE);
    ret.push(layer);
    ret
}

/// Check if a cached layer matches its digest
fn check_layer_digest(layer: &str) -> Result<bool> {
    let path = get_layer_path(layer);
    Ok(path.exists() && try_digest(path).map(|d| format!("sha256:{}", d) == layer)?)
}

/// Pull a fs layer from registry (if necessary)
pub fn pull_layer(api: &oci::Api, name: &str, layer: &oci::api::Blob) -> Result<()> {
    println!("Pulling fs layer {layer}...", layer = layer.digest);

    // Check whether layer is already cached
    if check_layer_digest(&layer.digest)? {
	// Nothing left to be done
        return Ok(());
    }

    create_dir_all(PathBuf::from(FS_LAYERS_STORE))?;

    api.download_layer(get_layer_path(&layer.digest), name, layer)?;

    Ok(())
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
    let config = api.get_config(&container_name, &manifest.config)?;

    for l in manifest.layers {
	pull_layer(&api, &container_name, &l)?;
    }

    println!("{:?}", config);

    Ok(())
}
