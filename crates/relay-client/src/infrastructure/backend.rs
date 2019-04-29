use crate::errors::relay_error::RelayError;
use crate::infrastructure::backend::mock_backend::MockBackend;
use crate::infrastructure::backend::websocket_backend::WebSocketBackend;
use crate::infrastructure::managed_connection::ManagedConnection;
use crate::infrastructure::relay_event::RelayEvent;
use crate::infrastructure::transaction_manager::TransactionManager;
use crossbeam::crossbeam_channel;
use futures::future::Either;
use futures::Future;

pub(crate) mod mock_backend;
pub(crate) mod websocket_backend;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackendType {
    Mock,
    WebSocket,
}

pub struct Backend {
    target: BackendType,
    connection: ManagedConnection,
    channel: crossbeam_channel::Receiver<RelayEvent>,
}

pub struct BackendOptions {
    pub target: BackendType,
    pub remote: String,
    pub transaction_manager: TransactionManager,
}

impl Backend {
    /// Create a new backend with a named implementation
    pub fn new(options: BackendOptions) -> impl Future<Item = Backend, Error = RelayError> {
        let (sx, rx) = crossbeam_channel::unbounded();
        let promise = match options.target {
            BackendType::Mock => Either::A(MockBackend::new(options.transaction_manager.clone(), true)),
            BackendType::WebSocket => Either::B(WebSocketBackend::new(&options.remote, options.transaction_manager.clone(), sx)),
        };
        return promise.then(move |r| {
            println!("Resolver resolved: ok: {}", r.is_ok());
            match r {
                Ok(handler) => Ok(Backend {
                    channel: rx,
                    target: options.target,
                    connection: ManagedConnection::new(handler, options.transaction_manager.clone()),
                }),
                Err(e) => Err(e),
            }
        });
    }

    /// Return the target for this backend
    pub fn target(&self) -> BackendType {
        self.target
    }

    /// Return a channel for this backend
    pub fn channel(&self) -> crossbeam_channel::Receiver<RelayEvent> {
        return self.channel.clone();
    }

    /// Send an external event
    pub fn send(&self, event: RelayEvent) -> impl Future<Item = (), Error = RelayError> {
        self.connection.send(event)
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::backend::{Backend, BackendOptions};
    use crate::infrastructure::testing::block_on_future;
    use crate::infrastructure::transaction_manager::TransactionManager;
    use crate::BackendType;

    #[test]
    fn test_create_mock_backend() {
        let backend = block_on_future(Backend::new(BackendOptions {
            remote: format!("localhost:9977"),
            target: BackendType::Mock,
            transaction_manager: TransactionManager::new(),
        }))
        .unwrap();
        assert_eq!(backend.target(), BackendType::Mock);
    }
}
