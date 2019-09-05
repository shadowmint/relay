use crate::errors::relay_error::RelayError;

use crate::{AuthOptions};
use chrono::Utc;
use relay_auth::AuthHasher;
use relay_auth::{AuthRequest, AuthSecretProvider};


pub struct AuthHelper {
    secret: String,
}

impl AuthHelper {
    pub fn generate_auth(options: &AuthOptions) -> Result<AuthRequest, RelayError> {
        let helper = AuthHelper {
            secret: options.secret.clone(),
        };
        let mut request = AuthRequest {
            expires: Utc::now().timestamp() + options.session_expires_secs,
            key: options.key.clone(),
            hash: None,
        };
        match AuthHasher::new().hash(&request, &helper) {
            Ok(h) => {
                request.hash = Some(h);
            }
            Err(e) => return Err(RelayError::AuthFailed(e)),
        }
        Ok(request)
    }
}

impl AuthSecretProvider for AuthHelper {
    fn secret_for(&self, _key: &str) -> Option<String> {
        Some(self.secret.to_string())
    }
}
