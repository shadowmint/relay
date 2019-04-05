use relay_auth::AuthSecretProvider;
use crate::ServerConfig;
use std::collections::HashMap;

pub struct ServerAuth {
    secrets: HashMap<String, String>
}

impl ServerAuth {
    pub fn new(config: &ServerConfig) -> ServerAuth {
        ServerAuth {
            secrets: config.secrets.clone()
        }
    }
}

impl AuthSecretProvider for ServerAuth {
    fn secret_for(&self, key: &str) -> Option<String> {
        self.secrets.get(key).map(|i| i.clone())
    }
}