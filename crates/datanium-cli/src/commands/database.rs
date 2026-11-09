use std::error::Error;

use clap::{Args, Subcommand};
use datanium_storage::DatabaseStore;

#[derive(Subcommand)]
pub enum DatabaseCommand {
    /// Create a database.
    Create(CreateArgs),

    /// List available databases.
    List,

    /// Opens a database.
    Open(OpenArgs),
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
            Self::Open(args) => {
                let database = store.open_database(&args.name)?;
                crate::session::run(database.name())?;
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

#[derive(Args)]
#[command(after_help = "Example:\n datanium database open example")]
pub struct OpenArgs {
    /// Name of the database.
    pub name: String,
}
