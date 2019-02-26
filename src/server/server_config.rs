use serde::{Serialize, Deserialize};
use toml::de::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    /// Bind to this address
    pub bind: String
}

impl ServerConfig {
    fn try_from(raw: &str) -> Result<ServerConfig, Error> {
        toml::from_str(raw)
    }
}