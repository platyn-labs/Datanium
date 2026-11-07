mod commands;

use clap::{Parser, Subcommand};

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

fn main() -> std::process::ExitCode {
    use std::process::ExitCode;

    let result = match Cli::parse().command {
        Command::Database(command) => command.run(),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprint!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}
