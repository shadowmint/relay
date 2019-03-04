use serde::{Serialize, Deserialize};
use crate::model::master_metadata::MasterMetadata;
use crate::model::external_error::ExternalError;
use rust_isolate::IsolateIdentity;

#[derive(Debug)]
pub enum MasterInternalEvent {
    /// A request from a client to join this master
    ClientJoinRequest { transaction_id: String, client_id: String, identity: IsolateIdentity },

    /// A client disconnected
    ClientDisconnected { identity: IsolateIdentity, reason: String },

    /// Send a message to the master
    MessageFromClient { transaction_id: String, client_id: IsolateIdentity, data: String },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
pub enum MasterExternalEvent {
    /// Sent by the client application to initialize a new session
    InitializeMaster { transaction_id: String, metadata: MasterMetadata },

    /// Recv a message from the external master to send to a client
    MessageToClient { transaction_id: String, client_id: String, data: String },

    /// Sent by the application to notify about transaction result
    TransactionResult { transaction_id: String, success: bool, error: Option<ExternalError> },

    /// Notify the master that a client joined
    ClientJoined { client_id: String, name: String },

    /// A client disconnected for some reason, a notification for the external master
    ClientDisconnected { client_id: String, reason: String },

    /// Send a message to the external master
    MessageFromClient { client_id: String, data: String },
}

#[derive(Debug)]
pub enum MasterControlEvent {
    /// Unconditionally halt immediately
    Halt,

    /// Sent by the websocket to notify of a master disconnect
    MasterDisconnected { reason: String },
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