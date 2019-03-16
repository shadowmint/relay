use rust_isolate::IsolateRegistryError;
use std::error::Error;
use std::fmt::Display;
use std::fmt;
use rust_isolate::IsolateRuntimeError;
use crossbeam::SendError;
use relay_core::events::master_event::MasterEvent;
use relay_core::events::client_event::ClientEvent;
use relay_analytics::analytics_error::AnalyticsError;

#[derive(Debug)]
pub enum ServerError {
    Failed(String)
}

impl Error for ServerError {}

impl Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<IsolateRegistryError> for ServerError {
    fn from(err: IsolateRegistryError) -> Self {
        ServerError::Failed(err.description().to_string())
    }
}

impl From<IsolateRuntimeError> for ServerError {
    fn from(err: IsolateRuntimeError) -> Self {
        ServerError::Failed(err.description().to_string())
    }
}

impl From<SendError<MasterEvent>> for ServerError {
    fn from(err: SendError<MasterEvent>) -> Self {
        ServerError::Failed(err.description().to_string())
    }
}

impl From<SendError<ClientEvent>> for ServerError {
    fn from(err: SendError<ClientEvent>) -> Self {
        ServerError::Failed(err.description().to_string())
    }
}

impl From<AnalyticsError> for ServerError {
    fn from(err: AnalyticsError) -> Self {
        ServerError::Failed(err.description().to_string())
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(err: serde_json::Error) -> Self {
        ServerError::Failed(err.description().to_string())
    }
}

impl From<ws::Error> for ServerError {
    fn from(err: ws::Error) -> Self {
        ServerError::Failed(err.description().to_string())
    }
}