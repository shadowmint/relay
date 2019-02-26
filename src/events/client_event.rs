use serde::{Serialize, Deserialize};
use crate::model::client_metadata::ClientMetadata;
use crate::model::external_error::ExternalError;

#[derive(Debug)]
pub enum ClientInternalEvent {
    /// The response from the master when a request is made join
    ClientJoinResponse { success: bool, error: Option<ExternalError> },

    /// Send a message to the master
    MessageFromMaster { transaction_id: String, format: String, data: String },

    /// Something went wrong with a request
    MessageFromClientResponse { transaction_id: String, success: bool, error: Option<ExternalError> },

    /// The master disconnected for some reason; this is a session ender
    MasterDisconnected { reason: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientExternalEvent {
    /// Sent by the client application to initialize a new session
    InitializeClient(ClientMetadata),

    /// Sent by the application to notify about initialization state (ready, error, etc)
    InitializeClientResponse { success: bool, error: Option<ExternalError> },

    /// Join a game by id
    Join { game_name: String },

    /// Sent by the application to notify about a join result
    JoinResponse { success: bool, error: Option<ExternalError> },

    /// Send a message to the master, this is a fire and forget action
    MessageFromClient { transaction_id: String, format: String, data: String },

    /// Recv a message from the master
    MessageToClient { transaction_id: String, format: String, data: String },

    /// The external client disconnected
    ClientDisconnected { reason: String },

    /// The internal master disconnected or booted this client
    MasterDisconnected { reason: String }
}

#[derive(Debug)]
pub enum ClientControlEvent {
    /// Unconditionally halt immediately
    Halt
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