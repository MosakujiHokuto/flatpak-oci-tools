use clap::{Parser, Subcommand};
use std::error::Error;

mod commands;
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
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::ImportContainer(args) => commands::import_container::run(args),
        Commands::ObsFetch(args) => commands::obs_fetch::run(args),
    }
}
