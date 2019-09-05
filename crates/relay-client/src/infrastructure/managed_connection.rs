use crate::errors::relay_error::RelayError;
use crate::infrastructure::relay_event::RelayEvent;
use crate::infrastructure::transaction_manager::TransactionManager;
use futures::future::Either;
use futures::{future, Future};
use std::sync::{Arc, Mutex};

pub trait ManagedConnectionHandler {
    fn send(&self, event: RelayEvent) -> Result<(), ()>;
}

#[derive(Clone)]
pub struct ManagedConnection {
    transactions: TransactionManager,
    internal: Arc<Mutex<Box<dyn ManagedConnectionHandler + Send + 'static>>>,
}

impl ManagedConnection {
    pub fn new(internal: Box<dyn ManagedConnectionHandler + Send + 'static>, transaction_manager: TransactionManager) -> ManagedConnection {
        ManagedConnection {
            transactions: transaction_manager,
            internal: Arc::new(Mutex::new(internal)),
        }
    }

    pub fn send(&self, event: RelayEvent) -> impl Future<Item = (), Error = RelayError> {
        match event.transaction_id() {
            Some(s) => Either::A(self.send_internal(&s, event)),
            None => Either::B(futures::failed(RelayError::InvalidEvent(format!("No transaction id found")))),
        }
    }

    fn send_internal(&self, transaction_id: &str, event: RelayEvent) -> impl Future<Item = (), Error = RelayError> {
        match self.internal.lock() {
            Ok(internal) => match internal.send(event) {
                Ok(_) => return Either::A(self.transactions.defer(transaction_id)),
                Err(_) => return Either::B(ManagedConnection::internal_error()),
            },
            Err(_) => return Either::B(ManagedConnection::internal_error()),
        }
    }

    fn internal_error() -> impl Future<Item = (), Error = RelayError> {
        return future::err::<(), RelayError>(RelayError::InternalError(format!("Send failed")));
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::backend::mock_backend::MockBackend;
    use crate::infrastructure::managed_connection::ManagedConnection;
    use crate::infrastructure::relay_event::RelayEvent;
    use crate::infrastructure::testing::block_on_future;
    use crate::infrastructure::transaction_manager::TransactionManager;
    use futures::future::Either;
    use futures::Future;
    use relay_auth::{AuthEvent, AuthRequest};
    use relay_core::events::master_event::MasterExternalEvent;
    use relay_core::model::master_metadata::MasterMetadata;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_create_managed_connection() {
        let manager = TransactionManager::new();
        let resolver = manager.clone();
        let mock = block_on_future(MockBackend::new(manager.clone(), false)).unwrap();

        let connection = ManagedConnection::new(mock, manager);

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            resolver.resolve("1234", Ok(())).unwrap();
        });
    }

    #[test]
    fn test_chain_events() {
        let manager = TransactionManager::new();
        let resolver = manager.clone();
        let mock = block_on_future(MockBackend::new(resolver.clone(), false)).unwrap();

        let connection = ManagedConnection::new(mock, manager);
        let connection_ref = connection.clone();
        let promise = connection.send(RelayEvent::Master(MasterExternalEvent::InitializeMaster {
            transaction_id: "5678".to_string(),
            metadata: MasterMetadata {
                master_id: "hello".to_string(),
                max_clients: 123,
            },
        }));

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            resolver.resolve("1234", Ok(())).unwrap();
            thread::sleep(Duration::from_millis(10));
            resolver.resolve("5678", Ok(())).unwrap();
        });

        let result = block_on_future(promise);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transaction_timeout() {
        let mut manager = TransactionManager::new();
        manager.set_timeout(100, 10);

        let resolver = manager.clone();
        let mock = block_on_future(MockBackend::new(resolver.clone(), false)).unwrap();

        let connection = ManagedConnection::new(mock, manager);

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            resolver.resolve("1234", Ok(())).unwrap();
        });
    }

    #[test]
    fn test_transaction_with_timeout_can_still_pass() {
        let mut manager = TransactionManager::new();
        manager.set_timeout(500, 10);

        let resolver = manager.clone();
        let mock = block_on_future(MockBackend::new(resolver.clone(), false)).unwrap();

        let connection = ManagedConnection::new(mock, manager);

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            resolver.resolve("1234", Ok(())).unwrap();
        });
    }
}
