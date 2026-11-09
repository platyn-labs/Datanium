use std::error::Error;

use clap::{Args, Subcommand};
use datanium_storage::DatabaseStore;

#[derive(Subcommand)]
pub enum DatabaseCommand {
    /// Create a database.
    Create(CreateArgs),

    /// List available databases.
    List,
}

impl DatabaseCommand {
    pub fn run(self, store: &DatabaseStore) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Create(args) => {
                let database = store.create_empty_database(&args.name)?;
                println!("Created database '{}'.", database.name());
            }
            Self::List => {
                let mut databases = store.load_databases()?;
                databases.sort_unstable_by(|a, b| a.name().cmp(b.name()));

                for database in databases {
                    println!("{}", database.name());
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
