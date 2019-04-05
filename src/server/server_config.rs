use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs;
use crate::server::server_error::ServerError;
use std::error::Error;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConfig {
    /// Bind to this address
    pub bind: String,

    /// Set of key -> secret bindings
    pub secrets: HashMap<String, String>,
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