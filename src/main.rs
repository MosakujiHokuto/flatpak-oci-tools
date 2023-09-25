use clap::{Parser, Subcommand};
use std::error::Error;
use std::sync::OnceLock;

mod commands;
mod download;
mod exec;
mod flatpak;
mod obs;
mod oci;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

static IS_USER: OnceLock<bool> = OnceLock::new();

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(long)]
    user: bool,
}

#[derive(Subcommand)]
enum Commands {
    ImportContainer(commands::import_container::Args),
    ObsFetch(commands::obs_fetch::Args),
    Pull(commands::pull::Args),
    Install(commands::install::Args),
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    IS_USER.set(cli.user).unwrap();

    match &cli.command {
        Commands::ImportContainer(args) => commands::import_container::run(args),
        Commands::ObsFetch(args) => commands::obs_fetch::run(args),
        Commands::Pull(args) => commands::pull::run(args),
	Commands::Install(args) => commands::install::run(args),
    }
}
