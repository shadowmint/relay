use crate::model::master_metadata::MasterMetadata;
use crate::infrastructure::services::SessionManager;
use rust_isolate::IsolateIdentity;
use crate::model::external_error::ErrorCode;
use crate::model::external_error::ExternalError;
use crate::events::master_event::MasterExternalEvent;
use crate::isolates::master::MasterEventDispatch;
use crate::isolates::master::MasterEventDispatch::DispatchExternal;
use std::collections::HashMap;
use rust_isolate::IsolateChannel;
use crate::events::client_event::ClientEvent;
use crate::isolates::master::MasterEventDispatch::DispatchToClient;
use crate::events::client_event::ClientInternalEvent::ClientJoinResponse;
use crate::events::client_event::ClientInternalEvent;
use crate::infrastructure::relay_logger::RelayLogger;

pub struct MasterState {
    name: String,
    logger: RelayLogger,
    identity: IsolateIdentity,
    active: bool,
    metadata: Option<MasterMetadata>,
    clients: HashMap<IsolateIdentity, IsolateChannel<ClientEvent>>,
    manager: SessionManager,
}

impl MasterState {
    pub fn new(identity: IsolateIdentity, manager: SessionManager, logger: RelayLogger) -> MasterState {
        MasterState {
            logger,
            manager,
            identity,
            name: String::new(),
            clients: HashMap::new(),
            active: false,
            metadata: None,
        }
    }

    pub fn instance(&self, identity: IsolateIdentity, logger: RelayLogger) -> MasterState {
        MasterState {
            logger,
            manager: self.manager.clone(),
            identity,
            name: String::new(),
            clients: HashMap::new(),
            active: false,
            metadata: None,
        }
    }

    /// Return a reference to the given client, if it is connected
    pub fn get_client(&self, identity: &IsolateIdentity) -> Option<&IsolateChannel<ClientEvent>> {
        self.clients.get(identity)
    }

    pub fn external_initialize(&mut self, metadata: MasterMetadata) -> MasterEventDispatch {
        match self.manager.register_session(&self.identity, &metadata.master_id) {
            Ok(_) => {
                self.name = metadata.master_id.clone();
                self.active = true;
                DispatchExternal(MasterExternalEvent::InitializeMasterResponse { success: true, error: None })
            }
            Err(e) => {
                DispatchExternal(MasterExternalEvent::InitializeMasterResponse { success: false, error: Some(ExternalError::from(e)) })
            }
        }
    }

    pub fn internal_client_join_request(&mut self, name: &str, identity: IsolateIdentity) -> Vec<MasterEventDispatch> {
        if self.clients.contains_key(&identity) {
            return vec!(
                DispatchToClient(identity, ClientJoinResponse { success: false, error: Some(ExternalError::from(ErrorCode::ClientIdConflict)) })
            );
        }
        if !self.active {
            return vec!(
                DispatchToClient(identity, ClientJoinResponse { success: false, error: Some(ExternalError::from(ErrorCode::NotActive)) })
            );
        }
        match self.metadata.as_ref() {
            Some(m) => {
                if self.clients.len() >= m.max_clients as usize {
                    return vec!(
                        DispatchToClient(identity, ClientJoinResponse { success: false, error: Some(ExternalError::from(ErrorCode::ClientLimitExceeded)) })
                    );
                }
            }
            _ => {}
        }
        match self.manager.find_client(&identity) {
            Ok(client_ref) => {
                self.clients.insert(identity.clone(), client_ref);
                vec!(
                    DispatchExternal(MasterExternalEvent::ClientJoined { name: name.to_string(), client: identity.to_string() }),
                    DispatchToClient(identity, ClientJoinResponse { success: true, error: None })
                )
            }
            Err(e) => {
                vec!(DispatchToClient(identity, ClientJoinResponse { success: false, error: Some(ExternalError::from(e)) }))
            }
        }
    }

    /// New message from some connected client
    pub fn internal_client_message(&self, client_id: IsolateIdentity, transaction_id: String, format: String, data: String) -> MasterEventDispatch {
        if !self.clients.contains_key(&client_id) {
            return DispatchToClient(client_id, ClientInternalEvent::MessageFromClientResponse {
                transaction_id,
                success: false,
                error: Some(ExternalError::from(ErrorCode::NoMatchingClientId)),
            });
        }

        DispatchExternal(MasterExternalEvent::MessageFromClient {
            client: self.identity.to_string(),
            transaction_id,
            format,
            data,
        })
    }

    /// A client disappeared
    pub fn internal_client_disconnected(&mut self, identity: IsolateIdentity, reason: &str) -> MasterEventDispatch {
        if self.clients.contains_key(&identity) {
            self.clients.remove(&identity);
        }
        self.logger.info(format!("Client disconnected: {}", reason));
        MasterEventDispatch::DispatchNone
    }

    /// New message from master to some connected client
    pub fn external_message_to_client(&self, client_id: String, transaction_id: String, format: String, data: String) -> MasterEventDispatch {
        // Attempt to resolve identity
        let identity = match IsolateIdentity::try_from(&client_id) {
            Ok(s) => s,
            Err(_) => {
                return DispatchExternal(MasterExternalEvent::MessageToClientResponse {
                    transaction_id,
                    success: false,
                    error: Some(ExternalError::from(ErrorCode::InvalidClientIdentityToken)),
                });
            }
        };

        // Check we know about that client
        if !self.clients.contains_key(&identity) {
            return DispatchExternal(MasterExternalEvent::MessageToClientResponse {
                transaction_id,
                success: false,
                error: Some(ExternalError::from(ErrorCode::NoMatchingClientId)),
            });
        }

        // If that all worked, send the message onwards
        DispatchToClient(identity, ClientInternalEvent::MessageFromMaster {
            transaction_id,
            format,
            data,
        })
    }

    /// The master itself disconnected for some reason.
    /// End the game session, notify all clients
    pub fn external_master_disconnected(&mut self, reason: &str) -> Vec<MasterEventDispatch> {
        self.active = false;

        // Collect notifications
        let mut notifications = Vec::new();
        self.clients.iter().for_each(|(k, _)| {
            notifications.push(MasterEventDispatch::DispatchToClient(k.clone(), ClientInternalEvent::MasterDisconnected {
                reason: reason.to_string()
            }))
        });

        self.logger.info(format!("Master disconnected: {}", reason));
        let _ = self.manager.remove_session(&self.name);
        return notifications;
    }
}