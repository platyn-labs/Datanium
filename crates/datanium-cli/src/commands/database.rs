use std::error::Error;

use clap::{Args, Subcommand};
use datanium_core::database::Database;

#[derive(Subcommand)]
pub enum DatabaseCommand {
    /// Create a database.
    Create(CreateArgs),

    /// List available databases.
    List,
}

impl DatabaseCommand {
    pub fn run(self) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Create(args) => create(args),
            Self::List => Err("Listing databases requires persistent storage.".into()),
        }
    }
}

fn create(args: CreateArgs) -> Result<(), Box<dyn Error>> {
    let database = Database::new(&args.name)?;
    println!("Created database '{}' in memory.", database.name());
    Ok(())
}

#[derive(Args)]
#[command(after_help = "Example:\n datanium database create example")]
pub struct CreateArgs {
    /// Name of the database.
    pub name: String,
}
