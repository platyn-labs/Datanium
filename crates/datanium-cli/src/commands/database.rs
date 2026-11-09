use std::error::Error;

use clap::{Args, Subcommand};
use datanium_client::DataniumClient;

#[derive(Subcommand)]
pub enum DatabaseCommand {
    /// Create a database.
    Create(CreateArgs),

    /// List available databases.
    List,
}

impl DatabaseCommand {
    pub fn run(self, client: &DataniumClient) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Create(args) => {
                let database = client.create_database(&args.name)?;
                println!("Created database '{}'.", database.name);
            }
            Self::List => {
                for database in client.list_databases()? {
                    println!("{}", database.name);
                }
            }
        }
        Ok(())
    }
}

#[derive(Args)]
#[command(after_help = "Example:\n datanium database create example")]
pub struct CreateArgs {
    /// Name of the database.
    pub name: String,
}
