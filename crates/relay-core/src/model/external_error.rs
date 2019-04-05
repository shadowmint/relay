use serde::{Serialize, Deserialize};
use crate::infrastructure::services::SessionManagerError;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ErrorCode {
    ArcMutexFailure = 1,
    MasterIdConflict,
    ClientIdConflict,
    ClientLimitExceeded,
    NoMatchingMasterId,
    InvalidClientIdentityToken,
    NoMatchingClientId,
    ClientNotConnected,
    NotActive,
    AuthFailed,
}

/// For sending external errors
#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalError {
    pub error_code: i32,
    pub error_reason: String,
}

impl From<ErrorCode> for ExternalError {
    fn from(code: ErrorCode) -> Self {
        ExternalError {
            error_code: code as i32,
            error_reason: match code {
                ErrorCode::MasterIdConflict => "The requested master id is already in use",
                ErrorCode::ClientIdConflict => "The requested client id is already in use",
                ErrorCode::NoMatchingMasterId => "No master found matching the requested id",
                ErrorCode::ClientLimitExceeded => "Too many connected clients, no free slots",
                ErrorCode::NoMatchingClientId => "No client found matching the requested id",
                ErrorCode::InvalidClientIdentityToken => "The client identity was malformed",
                ErrorCode::ClientNotConnected => "No active connection to a master exists yet for this client",
                ErrorCode::NotActive => "The specific target is not active",
                _ => "Internal error",
            }.to_string(),
        }
    }
}

impl From<SessionManagerError> for ExternalError {
    fn from(e: SessionManagerError) -> Self {
        return match e {
            SessionManagerError::MutexSyncError => ExternalError::from(ErrorCode::ArcMutexFailure),
            SessionManagerError::NameAlreadyInUse => ExternalError::from(ErrorCode::MasterIdConflict),
            SessionManagerError::NoMatchingMaster => ExternalError::from(ErrorCode::NoMatchingMasterId),
            SessionManagerError::NoMatchingClient => ExternalError::from(ErrorCode::NoMatchingClientId),
        };
    }
}
