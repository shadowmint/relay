use crate::events::auth_event::AuthEvent;
use relay_core::model::external_error::ErrorCode::AuthFailed;
use relay_core::model::external_error::ExternalError;
use relay_logging::RelayLogger;
use std::error::Error;
use crate::infrastructure::validator::AuthValidator;
use crate::AuthProviderConfig;

pub enum AuthResponse {
    Failed(AuthEvent),
    Passed { event: AuthEvent, expires: i64 },
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
        match serde_json::from_str::<AuthEvent>(message) {
            Ok(event) => self.process_authorize_request(event),
            Err(err) => {
                self.logger.warn(format!("Failed to deserialize message: {}: {}", message, err.description()));
                AuthResponse::Failed(AuthEvent::TransactionResult {
                    transaction_id: "".to_string(),
                    success: false,
                    error: Some(ExternalError::from(AuthFailed)),
                })
            }
        }
    }

    /// Process an auth event and return a result or an error
    /// Returns an event to send to the client, and true/false for 'should keep connection'
    /// If 'should keep connection' is false,
    fn process_authorize_request(&self, request: AuthEvent) -> AuthResponse {
        match request {
            AuthEvent::Auth { request, transaction_id } => {
                let expires = request.expires;
                let key = request.key.clone();
                match self.validator.validate(&transaction_id.to_string(), request, &self.config) {
                    Ok(_) => {
                        self.logger.info(format!("Auth success: {}: key {}, expires: {}", transaction_id, key, expires));
                        AuthResponse::Passed {
                            expires,
                            event: AuthEvent::TransactionResult {
                                transaction_id,
                                success: true,
                                error: None,
                            },
                        }
                    }
                    Err(err) => {
                        self.logger.warn(format!("Auth attempt failed: {:?}", err));
                        AuthResponse::Failed(AuthEvent::TransactionResult {
                            transaction_id,
                            success: false,
                            error: Some(ExternalError::from(AuthFailed)),
                        })
                    }
                }
            }
            _ => {
                self.logger.warn(format!("Auth attempt failed: Invalid request: {:?}", request));
                AuthResponse::Failed(AuthEvent::TransactionResult {
                    transaction_id: "".to_string(),
                    success: false,
                    error: Some(ExternalError::from(AuthFailed)),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::mocks::MockAuthProviderConfig;
    use crate::AuthProvider;
    use crate::auth_provider::AuthResponse;
    use crate::events::auth_event::{AuthEvent, AuthRequest};
    use crate::infrastructure::hasher::AuthHasher;
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
            AuthResponse::Failed(_) => {}
            _ => unreachable!()
        }

        // Bad hash
        let request = serde_json::to_string(&AuthEvent::Auth {
            transaction_id: "123".to_string(),
            request: AuthRequest {
                expires: 123213,
                key: "12323".to_string(),
                hash: None,
            },
        }).unwrap();
        match auth.authorize(&request) {
            AuthResponse::Failed(_) => {}
            _ => unreachable!()
        }
    }

    #[test]
    fn test_valid_auth_passes() {
        let mut mocks = MockAuthProviderConfig::mock_config_with_secrets(
            vec!(("12345678".to_string(), "99998888".to_string()))
        );

        // Build a valid hash
        let mut request = AuthRequest {
            expires: Utc::now().timestamp() + 1600,
            key: "12345678".to_string(),
            hash: None,
        };
        request.hash = Some(AuthHasher::new().hash("1234567890", &request, mocks.secret_store.as_mut()).unwrap());

        // Build a valid request
        let event = AuthEvent::Auth { request, transaction_id: "1234567890".to_string() };
        let raw_event = serde_json::to_string(&event).unwrap();

        // Setup auth provider and check the request
        let auth = AuthProvider::new(mocks);
        match auth.authorize(&raw_event) {
            AuthResponse::Passed { event, expires } => {
                match event {
                    AuthEvent::TransactionResult { transaction_id, success, error } => {
                        assert!(success);
                        assert!(error.is_none());
                        assert!(expires > Utc::now().timestamp());
                        assert_eq!(transaction_id, "1234567890");
                    }
                    _ => unreachable!()
                }
            }
            _ => unreachable!()
        }
    }
}
