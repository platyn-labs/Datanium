use std::{env, process::ExitCode};

use datanium_storage::DatabaseStore;

use crate::catalog::DatabaseCatalog;

mod api;
mod catalog;

#[tokio::main]
async fn main() -> ExitCode {
    let data_dir = match env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Could not determine the data directory: {error}");
            return ExitCode::FAILURE;
        }
    };

    let store = DatabaseStore::new(data_dir.clone());
    let catalog = match DatabaseCatalog::open(store) {
        Ok(catalog) => catalog,
        Err(error) => {
            eprintln!(
                "Could not load databases from {}: {error:?}",
                data_dir.display()
            );
            return ExitCode::FAILURE;
        }
    };

    println!("Data directory: {}", data_dir.display());
    for name in catalog.list_databases() {
        println!("{name}")
    }

    let bind = env::var("DATANIUM_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = match tokio::net::TcpListener::bind(bind.as_str()).await {
        Ok(listener) => listener,
        Err(error) => {
            eprint!("Could not listen on {bind}: {error}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(error) = axum::serve(listener, api::router(catalog)).await {
        eprint!("Server failed: {error}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
