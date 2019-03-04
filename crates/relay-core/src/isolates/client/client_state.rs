use crate::infrastructure::services::SessionManager;
use rust_isolate::IsolateIdentity;
use crate::model::external_error::ErrorCode;
use crate::model::external_error::ExternalError;
use crate::model::client_metadata::ClientMetadata;
use crate::isolates::client::ClientEventDispatch;
use crate::isolates::client::ClientEventDispatch::DispatchExternal;
use crate::events::client_event::ClientExternalEvent;
use crate::events::master_event::MasterEvent;
use rust_isolate::IsolateChannel;
use crate::events::master_event::MasterInternalEvent;
use crate::isolates::client::ClientEventDispatch::DispatchInternal;

pub struct ClientState {
    name: String,
    identity: IsolateIdentity,
    active: bool,
    connected: bool,
    master: Option<IsolateChannel<MasterEvent>>,
    manager: SessionManager,
}

impl ClientState {
    pub fn new(identity: IsolateIdentity, manager: SessionManager) -> ClientState {
        ClientState {
            manager,
            identity,
            name: String::new(),
            master: None,
            active: false,
            connected: false,
        }
    }

    pub fn instance(&self, identity: IsolateIdentity) -> ClientState {
        ClientState {
            manager: self.manager.clone(),
            identity,
            master: None,
            name: String::new(),
            active: false,
            connected: false,
        }
    }

    /// External initialize
    pub fn external_initialize(&mut self, transaction_id: String, metadata: ClientMetadata) -> ClientEventDispatch {
        self.name = metadata.name.clone();
        self.active = true;
        DispatchExternal(ClientExternalEvent::TransactionResult {
            transaction_id,
            success: true,
            error: None,
        })
    }

    /// External request to join a master
    pub fn external_join(&mut self, transaction_id: String, master_id: &str) -> ClientEventDispatch {
        // First, lets see if we can lookup the session
        match self.manager.find_master(&master_id) {
            Ok(session_ref) => {
                // If we got the master, update to refer to it, and pass the request to join to the master
                self.master = Some(session_ref);
                DispatchInternal(MasterInternalEvent::ClientJoinRequest {
                    transaction_id,
                    client_id: self.name.clone(),
                    identity: self.identity.clone(),
                })
            }
            Err(e) => {
                DispatchExternal(ClientExternalEvent::TransactionResult {
                    transaction_id,
                    success: false,
                    error: Some(ExternalError::from(e)),
                })
            }
        }
    }

    /// External new message from the client
    pub fn external_message(&self, transaction_id: String, data: String) -> ClientEventDispatch {
        if !self.connected {
            return DispatchExternal(ClientExternalEvent::TransactionResult {
                transaction_id,
                success: false,
                error: Some(ExternalError::from(ErrorCode::ClientNotConnected)),
            });
        }

        DispatchInternal(MasterInternalEvent::MessageFromClient {
            transaction_id,
            client_id: self.identity.clone(),
            data,
        })
    }

    /// External disconnect message
    pub fn external_disconnect(&self, reason: &str) -> ClientEventDispatch {
        DispatchInternal(MasterInternalEvent::ClientDisconnected {
            identity: self.identity.clone(),
            reason: reason.to_string(),
        })
    }

    /// Response internally from a join request
    pub fn internal_join_response(&mut self, transaction_id: String, success: bool, error: Option<ExternalError>) -> ClientEventDispatch {
        if !success {
            return DispatchExternal(ClientExternalEvent::TransactionResult {
                transaction_id,
                success: false,
                error,
            });
        }

        self.connected = true;
        return DispatchExternal(ClientExternalEvent::TransactionResult {
            transaction_id,
            success: true,
            error: None,
        });
    }

    /// Response internally from a message to master request
    pub fn internal_message_response(&mut self, transaction_id: String, success: bool, error: Option<ExternalError>) -> ClientEventDispatch {
        return DispatchExternal(ClientExternalEvent::TransactionResult {
            transaction_id,
            success,
            error,
        });
    }

    /// Forward a message from the master to the external connection
    pub fn internal_message_from_master(&mut self, data: String) -> ClientEventDispatch {
        return DispatchExternal(ClientExternalEvent::MessageToClient {
            data,
        });
    }

    /// The master disconnected; for the message to the client before bailing
    pub fn internal_master_disconnect(&mut self, reason: &str) -> ClientEventDispatch {
        ClientEventDispatch::DispatchExternal(ClientExternalEvent::MasterDisconnected {
            reason: reason.to_string()
        })
    }

    /// Return a reference to the master channel if we have one
    pub fn master_ref(&self) -> Option<&IsolateChannel<MasterEvent>> {
        self.master.as_ref()
    }
}