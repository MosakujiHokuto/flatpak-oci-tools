use clap;
use sha256::try_digest;
use std::env;
use std::fs::create_dir_all;
use std::path::PathBuf;

use crate::oci;
use crate::Result;

// XXX make those customizable
const FS_LAYERS_STORE_GLOBAL: &str = "/var/lib/flatpak-oci-tools/layers";
const FS_LAYERS_STORE_USER: &str = ".local/share/flatpak-oci-tools/layers";

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
    let mut ret = if crate::IS_USER.get().unwrap().to_owned() {
	// Deprecated due to Windows related issues, not our problem
	#[allow(deprecated)]
	let mut r = env::home_dir().unwrap();
	r.push(FS_LAYERS_STORE_USER);
	r
    } else {
	PathBuf::from(FS_LAYERS_STORE_GLOBAL)
    };

    ret.push(layer);
    ret
}

/// Check if a cached layer matches its digest
fn check_layer_digest(layer: &str) -> Result<bool> {
    let path = get_layer_path(layer);
    Ok(path.exists() && try_digest(path).map(|d| format!("sha256:{}", d) == layer)?)
}

/// Pull a fs layer from registry (if necessary)
pub fn pull_layer(api: &oci::Api, name: &str, layer: &oci::Blob) -> Result<PathBuf> {
    println!("Pulling fs layer {layer}...", layer = layer.digest);
    let path = get_layer_path(&layer.digest);

    // check whether layer is already cached
    if !check_layer_digest(&layer.digest)? {
	// nope. pulling from registry
	create_dir_all(path.parent().unwrap())?;
	api.download_layer(get_layer_path(&layer.digest), name, layer)?;
    }

    Ok(path)
}

/// Pull all layers of a container from registry
pub fn pull_image(api: &oci::Api, container: &str, manifest: &oci::Manifest) -> Result<Vec<PathBuf>>
{
    let mut ret: Vec<PathBuf> = Vec::with_capacity(manifest.layers.len());

    for l in manifest.layers.iter() {
	ret.push(pull_layer(&api, container, &l)?);
    }

    Ok(ret)
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
    let manifest = api.get_manifest(container, tag)?;

    pull_image(&api, &container_name, &manifest)?;

    Ok(())
}
