use crate::infrastructure::managed_connection::ManagedConnectionHandler;
use futures::sync::oneshot;
use futures::sync::oneshot::Canceled;
use relay_auth::AuthError;
use relay_core::model::external_error::ExternalError;
use std::error::Error;
use std::fmt;
use std::sync::{MutexGuard, PoisonError};

#[derive(Debug, Clone)]
pub enum RelayError {
    InternalError(String),
    ConnectionFailed(String),
    SyncError(String),
    ArcMutexFailure,
    InvalidEvent(String),
    ExternalError(ExternalError),
    AuthFailed(AuthError),
    SerializationError(String),
    TransactionExpired,
}

impl Error for RelayError {}

impl fmt::Display for RelayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RelayClientError: {:?}", self)
    }
}

impl From<Canceled> for RelayError {
    fn from(e: Canceled) -> Self {
        RelayError::SyncError(e.description().to_string())
    }
}

impl From<serde_json::Error> for RelayError {
    fn from(e: serde_json::Error) -> Self {
        RelayError::SerializationError(e.description().to_string())
    }
}

impl From<ws::Error> for RelayError {
    fn from(e: ws::Error) -> Self {
        RelayError::ConnectionFailed(e.description().to_string())
    }
}

impl From<PoisonError<MutexGuard<'_, Option<oneshot::Sender<Result<Box<(dyn ManagedConnectionHandler + Send + 'static)>, RelayError>>>>>>
    for RelayError
{
    fn from(
        _: PoisonError<MutexGuard<'_, Option<oneshot::Sender<Result<Box<(dyn ManagedConnectionHandler + Send + 'static)>, RelayError>>>>>,
    ) -> Self {
        RelayError::ArcMutexFailure
    }
}
