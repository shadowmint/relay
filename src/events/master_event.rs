use serde::{Serialize, Deserialize};
use crate::model::master_metadata::MasterMetadata;
use crate::model::external_error::ExternalError;
use rust_isolate::IsolateIdentity;

#[derive(Debug)]
pub enum MasterInternalEvent {
    /// A request from a client to join this master
    ClientJoinRequest { client_id: String, identity: IsolateIdentity },

    /// A client disconnected
    ClientDisconnected { identity: IsolateIdentity, reason: String },

    /// Send a message to the master
    MessageFromClient { client: IsolateIdentity, transaction_id: String, format: String, data: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MasterExternalEvent {
    /// Sent by the client application to initialize a new session
    InitializeMaster(MasterMetadata),

    /// Sent by the application to notify about initialization state (ready, error, etc)
    InitializeMasterResponse { success: bool, error: Option<ExternalError> },

    /// Notify the master that a client joined
    ClientJoined { client: String, name: String },

    /// Send a message to the external master
    MessageFromClient { client: String, transaction_id: String, format: String, data: String },

    /// Recv a message from the external master to send to a client
    MessageToClient { client: String, transaction_id: String, format: String, data: String },

    /// The response to sending a message to a client
    MessageToClientResponse { transaction_id: String, success: bool, error: Option<ExternalError> },

    /// The master disconnected for some reason; this is a session ender
    MasterDisconnected { reason: String },
}

#[derive(Debug)]
pub enum MasterControlEvent {
    /// Unconditionally halt immediately
    Halt
}

#[derive(Debug)]
pub enum MasterEvent {
    /// An debug and test actions
    Control(MasterControlEvent),

    /// An external event from the external connection to a client application
    External(MasterExternalEvent),

    /// An internal event from some other internal service or event
    Internal(MasterInternalEvent),
}