use serde::{Serialize, Deserialize};
use relay_core::model::external_error::ExternalError;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
pub enum AuthEvent {
    /// Login request
    Auth { request: AuthRequest },

    /// Sent by the application to notify about transaction result
    TransactionResult { transaction_id: String, success: bool, error: Option<ExternalError> },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
pub struct AuthRequest {
    /// The transaction id to respond to the client with
    pub transaction_id: String,

    /// When this auth session will expire, it must be in the future and within the required
    /// configuration bounds for the auth provider.
    pub expires: i64,

    /// What key is this request being made with? It must be a valid key according to the
    /// auth configuration.
    pub key: String,

    /// A hash to prove that the client knows what the secret key for the given public key
    /// is; it's basically just sha256(transaction_id:expires:key:secret)
    pub hash: Option<String>,
}
