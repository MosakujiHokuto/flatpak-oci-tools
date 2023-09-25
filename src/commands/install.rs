use clap;
use std::env;
use std::path::PathBuf;

use crate::commands::pull::pull_image;
use crate::exec::{CheckedRun, flatpak};
use crate::flatpak;
use crate::oci;
use crate::Result;

const FS_REPO_GLOBAL: &str = "/var/lib/flatpak-oci-tools/repo";
const FS_REPO_USER: &str = ".local/share/flatpak-oci-tools/repo";

use crate::IS_USER;

fn get_repo_path() -> PathBuf {
    if IS_USER.get().unwrap().to_owned() {
        // Deprecated due to Windows related issues, not our problem
        #[allow(deprecated)]
        let mut r = env::home_dir().unwrap();
        r.push(FS_REPO_USER);
        r
    } else {
        PathBuf::from(FS_REPO_GLOBAL)
    }
}

fn get_repo_name() -> String {
    if IS_USER.get().unwrap().to_owned() {
	"oci-tools-user".to_string()
    } else {
	"oci-tools".to_string()
    }
}

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
    let manifest = api.get_manifest(&container_name, &tag)?;
    let config = api.get_config(&container_name, &manifest.config)?;

    println!("Pulling fs layers...");
    let layers = pull_image(&api, &container_name, &manifest)?;

    let appname = config
        .config
        .labels
        .get("org.opensuse.flatpak.appname")
        .ok_or("Missing label: org.opensuse.flatpak.appname")?;

    let runtime_id = format!("org.openSUSE.Platform.{appname}");
    let app_id = format!("org.openSUSE.App.{appname}");
    let arch = match config.architecture.as_str() {
        "amd64" => Ok("x86_64"),
        other => Err(format!("Unsupported architecture: {other}")),
    }?;
    let version = config
        .config
        .labels
        .get("org.opencontainers.image.version")
        .ok_or("Missing label: org.opencontainers.image.version")?;

    let repo = get_repo_path();

    flatpak::ensure_repo(repo.as_path())?;
    let builder = flatpak::Builder::new()?;

    println!("Building runtime");
    builder.build_runtime(repo.as_path(), layers, &runtime_id, arch, version)?;

    println!("Installing runtime");
    flatpak().arg("remote-add")
        .arg("--if-not-exists")
        .arg("--no-gpg-verify")
        .arg(get_repo_name())
        .arg(get_repo_path().as_os_str())
        .checked_run()?;
    flatpak().arg("install")
        .arg("--assumeyes")
        .arg(get_repo_name())
        .arg(format!("runtime/{runtime_id}/{arch}/{version}"))
        .checked_run()?;

    println!("Building application");
    builder.build_app(repo.as_path(), &app_id, &runtime_id, arch, version)?;

    println!("Installing application");
    flatpak().arg("install")
        .arg("--assumeyes")
        .arg(get_repo_name())
        .arg(format!("app/{app_id}/{arch}/master"))
        .checked_run()?;

    println!("Done.");

    Ok(())
}
