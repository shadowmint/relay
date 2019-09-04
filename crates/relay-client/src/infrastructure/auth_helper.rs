use crate::errors::relay_error::RelayError;
use crate::infrastructure::relay_event::RelayEvent;
use crate::{AuthOptions, MasterOptions};
use chrono::Utc;
use relay_auth::AuthHasher;
use relay_auth::{AuthEvent, AuthRequest, AuthSecretProvider};
use uuid::Uuid;

pub struct AuthHelper {
    secret: String,
}

impl AuthHelper {
    pub fn generate_auth(options: &AuthOptions) -> Result<AuthEvent, RelayError> {
        let helper = AuthHelper {
            secret: options.secret.clone(),
        };
        let mut request = AuthRequest {
            expires: Utc::now().timestamp() + options.session_expires_secs,
            key: options.key.clone(),
            hash: None,
        };
        let transaction_id = Uuid::new_v4().to_string();
        match AuthHasher::new().hash(&transaction_id, &request, &helper) {
            Ok(h) => {
                request.hash = Some(h);
            }
            Err(e) => return Err(RelayError::AuthFailed(e)),
        }

        Ok(AuthEvent::Auth { transaction_id, request })
    }
}

impl AuthSecretProvider for AuthHelper {
    fn secret_for(&self, key: &str) -> Option<String> {
        Some(self.secret.to_string())
    }
}
