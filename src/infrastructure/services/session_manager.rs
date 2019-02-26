use std::sync::Arc;
use std::sync::Mutex;
use crate::infrastructure::services::session_manager::session_manager_inner::SessionManagerInner;
use crate::infrastructure::services::SessionManagerError;
use rust_isolate::IsolateIdentity;
use rust_isolate::IsolateChannel;
use crate::events::master_event::MasterEvent;
use std::sync::PoisonError;
use std::sync::MutexGuard;
use rust_isolate::IsolateRegistryRef;
use crate::events::client_event::ClientEvent;

pub mod session_manager_error;
mod session_manager_inner;

#[derive(Clone)]
pub struct SessionManager {
    inner: Arc<Mutex<SessionManagerInner>>
}

impl SessionManager {
    pub fn new(registry: IsolateRegistryRef) -> SessionManager {
        SessionManager {
            inner: Arc::new(Mutex::new(SessionManagerInner::new(registry)))
        }
    }

    /// Register a new game, if there isn't a conflict in the requested name
    pub fn register_session(&self, identity: &IsolateIdentity, name: &str) -> Result<(), SessionManagerError> {
        let mut inner = self.inner.lock()?;
        inner.register_session(identity, name)
    }

    /// Remove an existing session
    pub fn remove_session(&mut self, name: &str) -> Result<(), SessionManagerError> {
        let mut inner = self.inner.lock()?;
        inner.remove_session(name)
    }


    /// Find a registered master by name
    pub fn find_master(&self, name: &str) -> Result<IsolateChannel<MasterEvent>, SessionManagerError> {
        let inner = self.inner.lock()?;
        let master_ref = inner.find_master(name)?;
        Ok(master_ref)
    }

    /// Find a registered game by name
    pub fn find_client(&self, identity: &IsolateIdentity) -> Result<IsolateChannel<ClientEvent>, SessionManagerError> {
        let inner = self.inner.lock()?;
        let client_ref = inner.find_client(identity)?;
        Ok(client_ref)
    }
}

impl From<PoisonError<MutexGuard<'_, SessionManagerInner>>> for SessionManagerError {
    fn from(_: PoisonError<MutexGuard<SessionManagerInner>>) -> Self {
        SessionManagerError::MutexSyncError
    }
}