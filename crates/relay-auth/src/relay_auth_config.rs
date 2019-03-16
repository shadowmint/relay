use std::collections::HashMap;

#[derive(Clone)]
pub struct RelayAuthConfig {
    /// This is an auth token list, the mapping should be claim -> private token
    pub claims: HashMap<String, String>,

    /// Should auth failures (eg. bad hash) be an error, or just discard the claim?
    pub strict: bool,

    /// The maximum creation age of a token before it no longer validates
    pub token_max_age: i64,

    /// The token expires in this many seconds from the initial timestamp
    pub token_expires_in: i64,
}

impl Default for RelayAuthConfig {
    fn default() -> Self {
        RelayAuthConfig {
            claims: HashMap::new(),
            strict: true,
            token_max_age: 600,
            token_expires_in: 6000,
        }
    }
}