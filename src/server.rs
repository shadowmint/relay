use crate::server::server_error::ServerError;
use crate::server::server_config::ServerConfig;
use std::sync::Arc;
use std::sync::Mutex;
use crate::server::server_connection_factory::ServerConnectionFactory;
use ws::listen;

pub mod server_config;
pub mod server_error;
pub mod server_connection;
pub mod server_connection_factory;
pub mod server_auth;

pub struct Server {}

impl Server {
    pub fn new() -> Server {
        Server {}
    }

    /// Run the server
    pub fn listen(&mut self, config: ServerConfig) -> Result<(), ServerError> {
        let inner = ServerConnectionFactory::new(config.clone())?;
        let factory = Arc::new(Mutex::new(inner));
        listen(&config.bind, move |out| {
            match factory.lock() {
                Ok(factory_ref) => {
                    match factory_ref.new_connection(Some(out)) {
                        Ok(connection) => connection,
                        Err(_) => panic!("Failed to spawn connection")
                    }
                }
                Err(_) => panic!("Factory runtime is poisoned")
            }
        })?;
        Ok(())
    }
}