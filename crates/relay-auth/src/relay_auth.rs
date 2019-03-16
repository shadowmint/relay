use crate::RelayAuthConfig;
use crate::RelayAuthError;
use crate::Token;
use crate::relay_hasher::RelayHasher;

pub struct RelayAuth {
    config: RelayAuthConfig,
    hasher: RelayHasher,
}

impl RelayAuth {
    pub fn new(config: RelayAuthConfig) -> RelayAuth {
        RelayAuth {
            config,
            hasher: RelayHasher::new(),
        }
    }

    /// Validate an AuthToken and return a list of valid claims.
    /// The config controls if bad claims are discarded or errors.
    pub fn load(&mut self, token: Token) -> Result<(), RelayAuthError> {
        Err(RelayAuthError::NotImplemented)
    }

    /// Check if a particular claim is valid.
    /// If the token has expired, the return is false.
    pub fn has_claim(&self, claim: &str) -> bool {
        false
    }

    /// Return true if the auth token currently loaded has expired
    pub fn expired(&self) -> bool {
        false
    }
}