use std::{error::Error, time::Duration};

use datanium_protocol::{CreateDatabaseRequest, DATABASES_PATH, DatabaseInfo, ErrorBody};
use reqwest::{
    Url,
    blocking::{Client, Response},
};

pub struct DataniumClient {
    http: Client,
    databases_url: Url,
}

impl DataniumClient {
    pub fn new(server_url: &str) -> Result<Self, Box<dyn Error>> {
        let base = Url::parse(server_url)?;
        if !matches!(base.scheme(), "http" | "https") || base.host_str().is_none() {
            return Err("Expected an HTTP or HTTPS server URL.".into());
        }

        Ok(Self {
            http: Client::builder().timeout(Duration::from_secs(5)).build()?,
            databases_url: base.join(DATABASES_PATH)?,
        })
    }

    pub fn create_database(&self, name: &str) -> Result<DatabaseInfo, Box<dyn Error>> {
        let response = self
            .http
            .post(self.databases_url.clone())
            .json(&CreateDatabaseRequest {
                name: name.to_owned(),
            })
            .send()?;

        Ok(check_response(response)?.json()?)
    }

    pub fn list_databases(&self) -> Result<Vec<DatabaseInfo>, Box<dyn Error>> {
        let response = self.http.get(self.databases_url.clone()).send()?;
        Ok(check_response(response)?.json()?)
    }
}

fn check_response(response: Response) -> Result<Response, Box<dyn Error>> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    match response.json::<ErrorBody>() {
        Ok(error) => Err(format!("{status} [{}] {}", error.code, error.message).into()),
        Err(_) => Err(format!("Server returned HTTP {status}").into()),
    }
}
