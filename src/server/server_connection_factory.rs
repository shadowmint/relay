use crate::server::server_error::ServerError;
use rust_isolate::IsolateRegistry;
use relay_core::infrastructure::services::SessionManager;
use relay_core::CLIENT;
use relay_core::MASTER;
use relay_core::isolates::client::ClientIsolate;
use relay_core::isolates::master::MasterIsolate;
use rust_isolate::IsolateRuntimeRef;
use relay_core::events::master_event::MasterEvent;
use relay_core::events::client_event::ClientEvent;
use relay_logging::RelayLogger;
use ws::Sender;
use crate::server::server_connection::ServerConnection;
use relay_analytics::AnalyticsService;
use relay_analytics::analytics::Analytics;
use crate::ServerConfig;
use relay_auth::{AuthProvider, AuthProviderConfig};
use crate::server::server_auth::ServerAuth;

pub struct ServerConnectionFactory {
    logger: RelayLogger,
    config: ServerConfig,
    pub manager: SessionManager,
    pub registry: IsolateRegistry,
    pub masters: IsolateRuntimeRef<MasterEvent>,
    pub clients: IsolateRuntimeRef<ClientEvent>,
}

impl ServerConnectionFactory {
    /// Create a new instance ready to go
    pub fn new(config: ServerConfig) -> Result<ServerConnectionFactory, ServerError> {
        let mut registry = IsolateRegistry::new();
        let manager = SessionManager::new(registry.as_ref());
        let clients = registry.bind(CLIENT, ClientIsolate::new(manager.clone()))?;
        let masters = registry.bind(MASTER, MasterIsolate::new(manager.clone()))?;
        AnalyticsService::bind(&mut registry)?;
        Ok(ServerConnectionFactory {
            logger: RelayLogger::new("Websocket"),
            config,
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
        let analytics = Analytics::new(self.registry.as_ref())?;
        let auth = self.new_auth();
        Ok(ServerConnection::new(out, masters, clients, analytics, self.logger.clone(), auth))
    }

    /// Create a new configured auth provider for the service to use
    fn new_auth(&self) -> AuthProvider {
        AuthProvider::new(AuthProviderConfig {
            min_key_length: 8,
            min_transaction_id_length: 8,
            max_token_expiry: 3600,
            secret_store: Box::new(ServerAuth::new(&self.config)),
        })
    }
}