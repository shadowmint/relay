use serde::{Serialize, Deserialize};
use crate::model::client_metadata::ClientMetadata;
use crate::model::external_error::ExternalError;

#[derive(Debug)]
pub enum ClientInternalEvent {
    /// The response from the master when a request is made join
    ClientJoinResponse { transaction_id: String, success: bool, error: Option<ExternalError> },

    /// Send a message to the master
    MessageFromMaster { data: String },

    /// Something went wrong with a request
    MessageFromClientResponse { transaction_id: String, success: bool, error: Option<ExternalError> },

    /// The master disconnected for some reason; this is a session ender
    MasterDisconnected { reason: String },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
pub enum ClientExternalEvent {
    /// Sent by the client application to initialize a new session
    InitializeClient { transaction_id: String, metadata: ClientMetadata },

    /// Join a session by id
    Join { transaction_id: String, session_id: String },

    /// Send a message to the master, this is a fire and forget action
    MessageFromClient { transaction_id: String, data: String },

    /// Sent by the application to notify about transaction result
    TransactionResult { transaction_id: String, success: bool, error: Option<ExternalError> },

    /// Recv a message from the master
    MessageToClient { data: String },

    /// The internal master disconnected or booted this client
    /// This is a notification event, not an action by the client.
    MasterDisconnected { reason: String },
}

#[derive(Debug)]
pub enum ClientControlEvent {
    /// Unconditionally halt immediately
    Halt,

    /// Sent by the websocket handler to notify that the client disconnected
    ClientDisconnected { reason: String },
}

#[derive(Debug)]
pub enum ClientEvent {
    /// An debug and test actions
    Control(ClientControlEvent),

    /// An external event from the external connection to a client application
    External(ClientExternalEvent),

    /// An internal event from some other internal service or event
    Internal(ClientInternalEvent),
}