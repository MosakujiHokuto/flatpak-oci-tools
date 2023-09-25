use clap::{Parser, Subcommand};
use std::error::Error;

mod commands;
mod download;
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
    ImportContainer(commands::import_container::Args),
    ObsFetch(commands::obs_fetch::Args),
    Pull(commands::pull::Args),
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::ImportContainer(args) => commands::import_container::run(args),
        Commands::ObsFetch(args) => commands::obs_fetch::run(args),
	Commands::Pull(args) => commands::pull::run(args),
    }
}
