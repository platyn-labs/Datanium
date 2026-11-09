use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{State, rejection::JsonRejection},
    http::StatusCode,
    routing::get,
};
use datanium_protocol::{CreateDatabaseRequest, DATABASES_PATH, DatabaseInfo, ErrorBody};
use datanium_storage::StorageError;
use tokio::sync::Mutex;

use crate::catalog::{CreateDatabaseError, DatabaseCatalog};

type CatalogState = Arc<Mutex<DatabaseCatalog>>;
type ApiError = (StatusCode, Json<ErrorBody>);

pub(crate) fn router(catalog: DatabaseCatalog) -> Router {
    Router::new()
        .route(DATABASES_PATH, get(list_databases).post(create_database))
        .with_state(Arc::new(Mutex::new(catalog)))
}

async fn list_databases(State(catalog): State<CatalogState>) -> Json<Vec<DatabaseInfo>> {
    let catalog = catalog.lock().await;
    let databases = catalog
        .list_databases()
        .map(|name| DatabaseInfo {
            name: name.to_owned(),
        })
        .collect();

    Json(databases)
}

async fn create_database(
    State(catalog): State<CatalogState>,
    payload: Result<Json<CreateDatabaseRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<DatabaseInfo>), ApiError> {
    let Json(request) = payload.map_err(|_| {
        api_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "Expected a JSON datanase name.",
        )
    })?;

    tokio::task::spawn_blocking(move || {
        let mut catalog = catalog.blocking_lock();
        catalog
            .create_database(&request.name)
            .map_err(|error| match error {
                CreateDatabaseError::AlreadyExists => api_error(
                    StatusCode::CONFLICT,
                    "database_already_exists",
                    "A database with that name already exists.",
                ),
                CreateDatabaseError::Storage(StorageError::InvalidDatabaseName) => api_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_database_name",
                    "The datbase name is invalid.",
                ),
                CreateDatabaseError::Storage(_) => internal_error(),
            })?;

        Ok((
            StatusCode::CREATED,
            Json(DatabaseInfo { name: request.name }),
        ))
    })
    .await
    .map_err(|_| internal_error())?
}

fn api_error(status: StatusCode, code: &str, message: &str) -> ApiError {
    (
        status,
        Json(ErrorBody {
            code: code.into(),
            message: message.into(),
        }),
    )
}

fn internal_error() -> ApiError {
    api_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal_error",
        "The server could not create the database.",
    )
}
