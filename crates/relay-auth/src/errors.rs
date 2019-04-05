use std::error::Error;
use std::fmt::Display;
use std::fmt;

#[derive(Debug)]
pub enum AuthError {
    InvalidKey,
    InvalidHash,
    InvalidExpiry,
    InvalidTransactionId,
    UnknownError(String),
    SerializationError(String),
}

impl Error for AuthError {}

impl Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<serde_json::error::Error> for AuthError {
    fn from(err: serde_json::error::Error) -> Self {
        AuthError::SerializationError(err.description().to_string())
    }
}