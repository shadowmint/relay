use sha2::{Sha256, Digest};
use crate::events::auth_event::{AuthRequest};
use crate::AuthError;
use crate::auth_secret_provider::AuthSecretProvider;

pub struct AuthHasher {}

impl AuthHasher {
    /// Create a new hash worker
    pub fn new() -> AuthHasher {
        AuthHasher {}
    }

    /// Generate a new hash for a request, ignoring the hash field
    pub fn hash(&self, request: &AuthRequest, secret_store: &AuthSecretProvider) -> Result<String, AuthError> {
        // prep, see auth_events.rs, the format is: transaction_id:expires:key:secret
        let secret = match secret_store.secret_for(&request.key) {
            Some(s) => s,
            None => {
                return Err(AuthError::InvalidKey);
            }
        };
        let input = format!("{}:{}:{}:{}", request.transaction_id, request.expires, request.key, secret);

        // execute
        let mut hasher = Sha256::new();
        hasher.input(input);
        return Ok(format!("{:x}", hasher.result()));
    }

    /// Validate the hash on a request.
    /// The result is either Ok(()) or a failure reason.
    pub fn validate(&self, request: &AuthRequest, secret_store: &AuthSecretProvider) -> Result<(), AuthError> {
        match request.hash.as_ref() {
            Some(s) => {
                let hash = self.hash(request, secret_store)?;
                if hash == *s {
                    return Ok(());
                }
                return Err(AuthError::InvalidHash);
            }
            None => {
                Err(AuthError::InvalidHash)
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::infrastructure::hasher::AuthHasher;
    use crate::events::auth_event::AuthRequest;
    use crate::infrastructure::mocks::MockSecretProvider;

    #[test]
    fn test_create_hasher() {
        let _ = AuthHasher::new();
    }

    #[test]
    fn test_generate_valid_hash() {
        let request = AuthRequest {
            transaction_id: "123".to_string(),
            expires: 123,
            key: "123".to_string(),
            hash: None,
        };

        let mut secrets = MockSecretProvider::new();
        secrets.set("123", "123");

        let hasher = AuthHasher::new();
        let hash = hasher.hash(&request, &mut secrets).unwrap();
        println!("Hash: {}", hash);
        assert!(hash.len() > 0);
    }

    #[test]
    fn test_validate_hash() {
        let mut request = AuthRequest {
            transaction_id: "123".to_string(),
            expires: 123,
            key: "123".to_string(),
            hash: None,
        };

        let mut secrets = MockSecretProvider::new();
        secrets.set("123", "123");

        let hasher = AuthHasher::new();
        let hash = hasher.hash(&request, &mut secrets).unwrap();

        // Invalid before hash is assigned
        assert!(hasher.validate(&request, &mut secrets).is_err());

        // Valid after hash is assigned
        request.hash = Some(hash);
        assert!(hasher.validate(&request, &mut secrets).is_ok());
    }
}