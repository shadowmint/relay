use crate::infrastructure::validator::AuthValidator;
use crate::{AuthProviderConfig, AuthRequest};
use relay_logging::RelayLogger;
use std::error::Error;

pub enum AuthResponse {
    Failed,
    Passed { expires: i64 },
}

pub struct AuthProvider {
    config: AuthProviderConfig,
    validator: AuthValidator,
    logger: RelayLogger,
}

impl AuthProvider {
    /// Create a new instance given a specific secret store
    pub fn new(config: AuthProviderConfig) -> AuthProvider {
        AuthProvider {
            config,
            validator: AuthValidator::new(),
            logger: RelayLogger::new("RelayAuth"),
        }
    }

    /// Convert a string into an event or an error
    pub fn authorize(&self, message: &str) -> AuthResponse {
        match serde_json::from_str::<AuthRequest>(message) {
            Ok(event) => self.process_authorize_request(event),
            Err(err) => {
                self.logger.warn(format!(
                    "Failed to deserialize message: {}: {}",
                    message,
                    err.description()
                ));
                AuthResponse::Failed
            }
        }
    }

    /// Process an auth event and return a result or an error
    /// Returns an event to send to the client, and true/false for 'should keep connection'
    /// If 'should keep connection' is false,
    fn process_authorize_request(&self, request: AuthRequest) -> AuthResponse {
        let expires = request.expires;
        let key = request.key.clone();
        match self.validator.validate(request, &self.config) {
            Ok(_) => {
                self.logger
                    .info(format!("Auth success: key {}, expires: {}", key, expires));
                AuthResponse::Passed { expires }
            }
            Err(err) => {
                self.logger.warn(format!("Auth attempt failed: {:?}", err));
                AuthResponse::Failed
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::auth_provider::AuthResponse;
    use crate::events::auth_event::AuthRequest;
    use crate::infrastructure::hasher::AuthHasher;
    use crate::infrastructure::mocks::MockAuthProviderConfig;
    use crate::AuthProvider;
    use chrono::Utc;

    #[test]
    fn test_create_auth() {
        let config = MockAuthProviderConfig::mock_config();
        let _ = AuthProvider::new(config);
    }

    #[test]
    fn test_invalid_auth_fails() {
        let auth = AuthProvider::new(MockAuthProviderConfig::mock_config());

        // Bad format
        match auth.authorize("...") {
            AuthResponse::Failed => {}
            _ => unreachable!(),
        }

        // Bad hash
        let request = serde_json::to_string(&AuthRequest {
            expires: 123213,
            key: "12323".to_string(),
            hash: None,
        })
        .unwrap();

        match auth.authorize(&request) {
            AuthResponse::Failed => {}
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_valid_auth_passes() {
        let mut mocks = MockAuthProviderConfig::mock_config_with_secrets(vec![(
            "12345678".to_string(),
            "99998888".to_string(),
        )]);

        // Build a valid hash
        let mut request = AuthRequest {
            expires: Utc::now().timestamp() + 1600,
            key: "12345678".to_string(),
            hash: None,
        };
        request.hash = Some(
            AuthHasher::new()
                .hash(&request, mocks.secret_store.as_mut())
                .unwrap(),
        );

        // Build a valid request
        let raw_event = serde_json::to_string(&request).unwrap();

        // Setup auth provider and check the request
        let auth = AuthProvider::new(mocks);
        match auth.authorize(&raw_event) {
            AuthResponse::Passed { expires } => {
                assert!(expires > Utc::now().timestamp());
            }
            _ => unreachable!(),
        }
    }
}
