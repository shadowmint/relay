use rust_isolate::IsolateIdentity;
use crate::infrastructure::services::SessionManagerError;
use rust_isolate::IsolateChannel;
use crate::events::master_event::MasterEvent;
use crate::MASTER;
use std::collections::HashMap;
use rust_isolate::IsolateRegistryError;
use rust_isolate::IsolateRegistryRef;
use crate::CLIENT;
use crate::events::client_event::ClientEvent;

pub struct SessionManagerInner {
    registry: IsolateRegistryRef,
    sessions: HashMap<String, IsolateIdentity>,
}

impl SessionManagerInner {
    pub fn new(registry: IsolateRegistryRef) -> SessionManagerInner {
        SessionManagerInner {
            registry,
            sessions: HashMap::new(),
        }
    }

    /// Register a new session, if there isn't a conflict in the requested name
    pub fn register_session(&mut self, identity: &IsolateIdentity, name: &str) -> Result<(), SessionManagerError> {
        if self.sessions.contains_key(name) {
            return Err(SessionManagerError::NameAlreadyInUse);
        }
        self.sessions.insert(name.to_string(), identity.clone());
        Ok(())
    }

    /// Remove an existing session
    pub fn remove_session(&mut self, name: &str) -> Result<(), SessionManagerError> {
        if !self.sessions.contains_key(name) {
            return Err(SessionManagerError::NoMatchingMaster);
        }
        self.sessions.remove(name);
        Ok(())
    }

    /// Find a registered master by name
    pub fn find_master(&self, name: &str) -> Result<IsolateChannel<MasterEvent>, SessionManagerError> {
        // Find the session
        let identity = self.sessions.get(name);
        if identity.is_none() {
            return Err(SessionManagerError::NoMatchingMaster);
        }

        // Find a reference in the registry
        let master_runtime = self.registry.find(MASTER)?;
        match master_runtime.find(identity.unwrap()) {
            Some(master_ref) => Ok(master_ref),
            None => Err(SessionManagerError::NoMatchingMaster)
        }
    }

    /// Find a registered client by name
    pub fn find_client(&self, identity: &IsolateIdentity) -> Result<IsolateChannel<ClientEvent>, SessionManagerError> {
        let client_runtime = self.registry.find(CLIENT)?;
        match client_runtime.find(identity) {
            Some(client_ref) => Ok(client_ref),
            None => Err(SessionManagerError::NoMatchingClient)
        }
    }
}

impl From<IsolateRegistryError> for SessionManagerError {
    fn from(_: IsolateRegistryError) -> Self {
        SessionManagerError::NoMatchingMaster
    }
}