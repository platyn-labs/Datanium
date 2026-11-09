mod commands;
mod session;

use clap::{Parser, Subcommand};
use datanium_client::DataniumClient;

use crate::commands::database::DatabaseCommand;

#[derive(Parser)]
#[command(name = "datanium", version, about = "Manage Datanium databases")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    #[arg(long, global = true, default_value = "http://127.0.0.1:8080")]
    pub server_url: String,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create and manage databases.
    #[command(subcommand)]
    Database(DatabaseCommand),
    /// Open an interactive session with a Datanium server.
    Connect {
        /// Base URL of the Datanium server.
        url: String,
    },
}

fn main() -> std::process::ExitCode {
    use std::process::ExitCode;

    let cli = Cli::parse();
    let result = match cli.command {
        Command::Database(command) => {
            DataniumClient::new(&cli.server_url).and_then(|client| command.run(&client))
        }
        Command::Connect { url } => session::run(&url),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}
