use crate::auth_secret_provider::AuthSecretProvider;

pub struct AuthProviderConfig {
    /// Min length for transaction ids
    pub min_transaction_id_length: usize,

    /// Min length for keys
    pub min_key_length: usize,

    /// No token can be allowed to exist for longer than this
    pub max_token_expiry: i64,

    /// The set of secrets for this server
    pub secret_store: Box<dyn AuthSecretProvider>,
}