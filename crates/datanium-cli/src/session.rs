use std::{
    error::Error,
    io::{self, Write},
    iter,
};

use clap::{Parser, Subcommand};
use datanium_client::DataniumClient;

use crate::commands::database::DatabaseCommand;

#[derive(Parser)]
#[command(name = "datanium")]
struct SessionCli {
    #[command(subcommand)]
    command: SessionCommand,
}

#[derive(Subcommand)]
enum SessionCommand {
    /// Create and manage databases.
    #[command(subcommand)]
    Database(DatabaseCommand),
    /// End the current session.
    Disconnect,
}

pub(crate) fn run(url: &str) -> Result<(), Box<dyn Error>> {
    let client = DataniumClient::new(url)?;
    client.list_databases()?;
    println!("Connected to {url}");

    let mut line = String::new();
    loop {
        print!("datanium> ");
        io::stdout().flush()?;
        line.clear();

        if io::stdin().read_line(&mut line)? == 0 {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }

        let args = iter::once("datanium").chain(line.split_whitespace());
        match SessionCli::try_parse_from(args) {
            Ok(cli) => match cli.command {
                SessionCommand::Database(command) => {
                    if let Err(error) = command.run(&client) {
                        eprintln!("Error: {error}");
                    }
                }
                SessionCommand::Disconnect => break,
            },
            Err(error) => error.print()?,
        }
    }

    println!("Disconnected.");
    Ok(())
}
