use std::collections::HashMap;

pub struct RelayAuthConfig {
    /// This is an auth token list, the mapping should be claim -> token
    pub claims: HashMap<String, String>,

    /// Should auth failures (eg. bad hash) be an error, or just discard the claim?
    pub strict: bool,
}

impl Default for RelayAuthConfig {
    fn default() -> Self {
        RelayAuthConfig {
            claims: HashMap::new(),
            strict: true,
        }
    }
}