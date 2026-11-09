mod commands;
mod session;

use std::{env, error::Error, process::ExitCode};

use clap::{Parser, Subcommand};
use datanium_storage::DatabaseStore;

use crate::commands::database::DatabaseCommand;

#[derive(Parser)]
#[command(name = "datanium", version, about = "Manage Datanium databases")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create and manage databases.
    #[command(subcommand)]
    Database(DatabaseCommand),
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Database(command) => {
            let store = DatabaseStore::new(env::current_dir()?);
            command.run(&store)
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}
