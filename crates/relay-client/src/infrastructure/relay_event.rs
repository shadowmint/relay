use relay_auth::AuthEvent;
use relay_core::events::client_event::ClientExternalEvent;
use relay_core::events::master_event::MasterExternalEvent;
use relay_core::model::external_error::{ErrorCode, ExternalError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
pub enum RelayEvent {
    Master(MasterExternalEvent),
    Client(ClientExternalEvent),
}

impl RelayEvent {
    pub fn transaction_id(&self) -> Option<String> {
        match self {
            RelayEvent::Master(m) => match m {
                MasterExternalEvent::ClientDisconnected { client_id: _, reason: _ } => None,
                MasterExternalEvent::InitializeMaster { transaction_id, metadata: _ } => Some(transaction_id.to_string()),
                MasterExternalEvent::MessageToClient {
                    transaction_id,
                    client_id: _,
                    data: _,
                } => Some(transaction_id.to_string()),
                MasterExternalEvent::TransactionResult {
                    transaction_id,
                    success: _,
                    error: _,
                } => Some(transaction_id.to_string()),
                MasterExternalEvent::ClientJoined { client_id: _, name: _ } => None,
                MasterExternalEvent::MessageFromClient { client_id: _, data: _ } => None,
            },
            RelayEvent::Client(c) => match c {
                ClientExternalEvent::InitializeClient { transaction_id, metadata: _ } => Some(transaction_id.to_string()),
                ClientExternalEvent::Join {
                    transaction_id,
                    session_id: _,
                } => Some(transaction_id.to_string()),
                ClientExternalEvent::MessageFromClient { transaction_id, data: _ } => Some(transaction_id.to_string()),
                ClientExternalEvent::TransactionResult {
                    transaction_id,
                    success: _,
                    error: _,
                } => Some(transaction_id.to_string()),
                ClientExternalEvent::MessageToClient { data: _ } => None,
                ClientExternalEvent::MasterDisconnected { reason: _ } => None,
            },
        }
    }

    pub fn transaction_result(&self) -> Result<(), ExternalError> {
        match self {
            RelayEvent::Master(m) => match m {
                MasterExternalEvent::ClientDisconnected { client_id: _, reason: _ } => Err(ExternalError::from(ErrorCode::Unknown)),
                MasterExternalEvent::InitializeMaster { transaction_id, metadata: _ } => Err(ExternalError::from(ErrorCode::Unknown)),
                MasterExternalEvent::MessageToClient {
                    transaction_id: _,
                    client_id: _,
                    data: _,
                } => Err(ExternalError::from(ErrorCode::Unknown)),
                MasterExternalEvent::TransactionResult {
                    transaction_id: _,
                    success,
                    error,
                } => {
                    if *success {
                        Ok(())
                    } else {
                        let err: Option<ExternalError> = error.clone();
                        Err(err.unwrap_or(ExternalError::from(ErrorCode::Unknown)))
                    }
                }
                MasterExternalEvent::ClientJoined { client_id: _, name: _ } => Err(ExternalError::from(ErrorCode::Unknown)),
                MasterExternalEvent::MessageFromClient { client_id: _, data: _ } => Err(ExternalError::from(ErrorCode::Unknown)),
            },
            RelayEvent::Client(c) => match c {
                ClientExternalEvent::InitializeClient { transaction_id, metadata: _ } => Err(ExternalError::from(ErrorCode::Unknown)),
                ClientExternalEvent::Join {
                    transaction_id: _,
                    session_id: _,
                } => Err(ExternalError::from(ErrorCode::Unknown)),
                ClientExternalEvent::MessageFromClient { transaction_id, data: _ } => Err(ExternalError::from(ErrorCode::Unknown)),
                ClientExternalEvent::TransactionResult {
                    transaction_id: _,
                    success,
                    error,
                } => {
                    if *success {
                        Ok(())
                    } else {
                        let err: Option<ExternalError> = error.clone();
                        Err(err.unwrap_or(ExternalError::from(ErrorCode::Unknown)))
                    }
                }
                ClientExternalEvent::MessageToClient { data: _ } => Err(ExternalError::from(ErrorCode::Unknown)),
                ClientExternalEvent::MasterDisconnected { reason: _ } => Err(ExternalError::from(ErrorCode::Unknown)),
            },
        }
    }
}
