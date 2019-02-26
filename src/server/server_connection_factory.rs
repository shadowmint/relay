use crate::server::server_error::ServerError;
use rust_isolate::IsolateRegistry;
use crate::infrastructure::services::SessionManager;
use crate::CLIENT;
use crate::MASTER;
use crate::isolates::client::ClientIsolate;
use crate::isolates::master::MasterIsolate;
use rust_isolate::IsolateRuntimeRef;
use crate::events::master_event::MasterEvent;
use crate::events::client_event::ClientEvent;
use crate::infrastructure::relay_logger::RelayLogger;
use ws::Sender;
use crate::server::server_connection::ServerConnection;

pub struct ServerConnectionFactory {
    logger: RelayLogger,
    pub manager: SessionManager,
    pub registry: IsolateRegistry,
    pub masters: IsolateRuntimeRef<MasterEvent>,
    pub clients: IsolateRuntimeRef<ClientEvent>,
}

impl ServerConnectionFactory {
    /// Create a new instance ready to go
    pub fn new() -> Result<ServerConnectionFactory, ServerError> {
        let mut registry = IsolateRegistry::new();
        let manager = SessionManager::new(registry.as_ref());
        let clients = registry.bind(CLIENT, ClientIsolate::new(manager.clone()))?;
        let masters = registry.bind(MASTER, MasterIsolate::new(manager.clone()))?;
        Ok(ServerConnectionFactory {
            logger: RelayLogger::new(None, "websocket"),
            manager,
            registry,
            masters,
            clients,
        })
    }

    /// Spawn a new connection to handle this connection
    pub fn new_connection(&self, out: Option<Sender>) -> Result<ServerConnection, ServerError> {
        let clients = self.registry.find::<ClientEvent>(CLIENT)?;
        let masters = self.registry.find::<MasterEvent>(MASTER)?;
        Ok(ServerConnection::new(out, masters, clients, self.logger.clone()))
    }
}