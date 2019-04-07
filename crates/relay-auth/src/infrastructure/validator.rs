use crate::infrastructure::hasher::AuthHasher;
use crate::events::auth_event::AuthRequest;
use crate::AuthError;
use chrono::Utc;
use crate::AuthProviderConfig;

pub struct AuthValidator {
    hasher: AuthHasher,
}

impl AuthValidator {
    /// Create a new validator
    pub fn
    new() -> AuthValidator {
        AuthValidator {
            hasher: AuthHasher::new(),
        }
    }

    /// Validate a request.
    /// The response is Ok(()) or a error with a reason
    pub fn validate(&self, transaction_id: &str, request: AuthRequest, config: &AuthProviderConfig) -> Result<(), AuthError> {
        self.validate_hash(transaction_id, &request, config)?;
        self.validate_transaction_id(transaction_id, config)?;
        self.validate_expires(&request, config)?;
        self.validate_key(&request, config)?;
        Ok(())
    }

    fn validate_hash(&self, transaction_id: &str, request: &AuthRequest, config: &AuthProviderConfig) -> Result<(), AuthError> {
        self.hasher.validate(transaction_id, request, config.secret_store.as_ref())?;
        Ok(())
    }

    fn validate_transaction_id(&self, transaction_id: &str, config: &AuthProviderConfig) -> Result<(), AuthError> {
        if transaction_id.len() < config.min_transaction_id_length {
            return Err(AuthError::InvalidTransactionId);
        }
        Ok(())
    }

    fn validate_expires(&self, request: &AuthRequest, config: &AuthProviderConfig) -> Result<(), AuthError> {
        let now = Utc::now().timestamp();
        let max_expires = now + config.max_token_expiry;
        if request.expires < now || request.expires > max_expires {
            return Err(AuthError::InvalidExpiry);
        }
        Ok(())
    }

    fn validate_key(&self, request: &AuthRequest, config: &AuthProviderConfig) -> Result<(), AuthError> {
        if request.key.len() < config.min_key_length {
            return Err(AuthError::InvalidKey);
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::infrastructure::mocks::{MockSecretProvider, MockAuthProviderConfig};
    use chrono::Utc;
    use crate::events::auth_event::AuthRequest;
    use crate::infrastructure::hasher::AuthHasher;
    use crate::AuthError;
    use crate::infrastructure::validator::AuthValidator;

    #[test]
    fn test_validate_request() {
        // Setup
        let mut secrets = MockSecretProvider::new();
        secrets.set("12321321321", "2121312313");

        let expires = Utc::now().timestamp() + 3600;

        let mut request = AuthRequest {
            key: "12321321321".to_string(),
            expires,
            hash: None,
        };

        // Sign request
        request.hash = Some(AuthHasher::new().hash("32dfjadkjladskladfadfasdf", &request, &mut secrets).unwrap());

        // Execute
        let mut config = MockAuthProviderConfig::mock_config_with_store(secrets);
        let validator = AuthValidator::new();

        // Assert
        assert!(validator.validate("32dfjadkjladskladfadfasdf", request, &mut config).is_ok());
    }

    #[test]
    fn test_invalid_request() {
        // Setup
        let mut secrets_wrong = MockSecretProvider::new();
        let mut secrets = MockSecretProvider::new();
        secrets_wrong.set("12321321321", "1111111111");
        secrets.set("12321321321", "2121312333");

        let expires = Utc::now().timestamp() + 3600;

        let mut request = AuthRequest {
            key: "12321321321".to_string(),
            expires,
            hash: None,
        };

        // Sign request
        request.hash = Some(AuthHasher::new().hash("32dfjadkjladskladfadfasdf", &request, &mut secrets_wrong).unwrap());

        // Execute
        let mut config = MockAuthProviderConfig::mock_config_with_store(secrets);
        let validator = AuthValidator::new();

        // Assert
        match validator.validate("32dfjadkjladskladfadfasdf", request, &mut config) {
            Ok(_) => unreachable!(),
            Err(e) => {
                match e {
                    AuthError::InvalidHash => {}
                    _ => unreachable!()
                }
            }
        }
    }

    #[test]
    fn test_create_hasher() {
        let _ = AuthValidator::new();
    }
}