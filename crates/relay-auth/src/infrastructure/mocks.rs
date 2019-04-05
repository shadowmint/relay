use std::collections::HashMap;
use crate::auth_secret_provider::AuthSecretProvider;
use crate::AuthProviderConfig;

pub struct MockSecretProvider {
    map: HashMap<String, String>
}

impl MockSecretProvider {
    pub fn new() -> MockSecretProvider {
        MockSecretProvider {
            map: HashMap::new()
        }
    }

    pub fn set(&mut self, key: &str, secret: &str) {
        self.map.insert(key.to_string(), secret.to_string());
    }

    pub fn set_many(&mut self, secrets: Vec<(String, String)>) {
        secrets.into_iter().for_each(|(k, v)| {
            self.map.insert(k, v);
        })
    }
}

impl AuthSecretProvider for MockSecretProvider {
    fn secret_for(&self, key: &str) -> Option<String> {
        if self.map.contains_key(key) {
            return Some(self.map[key].to_string());
        }
        None
    }
}

pub struct MockAuthProviderConfig {}

impl MockAuthProviderConfig {
    pub fn mock_config() -> AuthProviderConfig {
        AuthProviderConfig {
            secret_store: Box::new(MockSecretProvider::new()),
            min_transaction_id_length: 8,
            min_key_length: 8,
            max_token_expiry: 3600,
        }
    }

    pub fn mock_config_with_secrets(secrets: Vec<(String, String)>) -> AuthProviderConfig {
        let mut store = MockSecretProvider::new();
        store.set_many(secrets);
        AuthProviderConfig {
            secret_store: Box::new(store),
            min_transaction_id_length: 8,
            min_key_length: 8,
            max_token_expiry: 3600,
        }
    }

    pub fn mock_config_with_store(secrets: impl AuthSecretProvider + 'static) -> AuthProviderConfig {
        AuthProviderConfig {
            secret_store: Box::new(secrets),
            min_transaction_id_length: 8,
            min_key_length: 8,
            max_token_expiry: 3600,
        }
    }
}