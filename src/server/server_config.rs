use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs;
use crate::server::server_error::ServerError;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    /// Bind to this address
    pub bind: String
}

impl ServerConfig {
    pub fn try_from<T: AsRef<Path>>(path: T) -> Result<ServerConfig, ServerError> {
        let raw = fs::read_to_string(path)?;
        return Ok(toml::from_str(&raw)?);
    }
}

impl From<toml::de::Error> for ServerError {
    fn from(err: toml::de::Error) -> Self {
        ServerError::Failed(err.description().to_string())
    }
}

impl From<std::io::Error> for ServerError {
    fn from(err: std::io::Error) -> Self {
        ServerError::Failed(err.description().to_string())
    }
}