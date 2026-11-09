use serde::{Deserialize, Serialize};

pub const DATABASES_PATH: &str = "/v1/databases";

#[derive(Deserialize, Serialize)]
pub struct CreateDatabaseRequest {
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct DatabaseInfo {
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}
