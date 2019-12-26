use crate::analytics_context::AnalyticsContext;
use crate::AnalyticsEventType;
use base_logging::Loggable;
use futures::sync::oneshot;
use futures::sync::oneshot::Canceled;
use rust_isolate::IsolateRegistryError;
use rust_isolate::IsolateRuntimeError;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::sync::{MutexGuard, PoisonError};

#[derive(Debug)]
pub enum AnalyticsError {
    RuntimeError(IsolateRuntimeError),
    RegistryError(IsolateRegistryError),
    AsyncError(String),
    QueryError(String),
}

impl Error for AnalyticsError {}

impl Loggable for AnalyticsError {
    fn log_message(&self) -> Option<&str> {
        Some(self.description())
    }

    fn log_properties(&self) -> Option<HashMap<&str, &str>> {
        None
    }
}

impl Display for AnalyticsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<IsolateRuntimeError> for AnalyticsError {
    fn from(err: IsolateRuntimeError) -> Self {
        AnalyticsError::RuntimeError(err)
    }
}

impl From<IsolateRegistryError> for AnalyticsError {
    fn from(err: IsolateRegistryError) -> Self {
        AnalyticsError::RegistryError(err)
    }
}

impl From<oneshot::Canceled> for AnalyticsError {
    fn from(e: Canceled) -> Self {
        AnalyticsError::AsyncError(format!("{}", e))
    }
}

impl From<crossbeam::SendError<AnalyticsEventType>> for AnalyticsError {
    fn from(e: crossbeam::SendError<AnalyticsEventType>) -> Self {
        AnalyticsError::AsyncError(format!("{}", e))
    }
}

impl From<crossbeam::SendError<Vec<String>>> for AnalyticsError {
    fn from(e: crossbeam::SendError<Vec<String>>) -> Self {
        AnalyticsError::AsyncError(format!("{}", e))
    }
}

impl From<crossbeam::SendError<HashMap<String, i32>>> for AnalyticsError {
    fn from(e: crossbeam::SendError<HashMap<String, i32>>) -> Self {
        AnalyticsError::AsyncError(format!("{}", e))
    }
}

impl From<PoisonError<MutexGuard<'_, AnalyticsContext>>> for AnalyticsError {
    fn from(e: PoisonError<MutexGuard<AnalyticsContext>>) -> Self {
        AnalyticsError::AsyncError(format!("{}", e))
    }
}
