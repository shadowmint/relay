use crate::infrastructure::transaction_manager::transaction_manager_inner::TransactionManagerInner;
use futures::future::Either;
use futures::sync::oneshot;
use futures::Future;
use relay_core::model::external_error::ErrorCode::{ArcMutexFailure, SyncError};
use relay_core::model::external_error::ExternalError;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{Sender, channel, TryRecvError};
use std::time::Duration;
use crate::errors::relay_error::RelayError;
use std::error::Error;

mod transaction_manager_inner;

#[derive(Clone)]
pub struct TransactionManager {
    inner: Arc<Mutex<TransactionManagerInner>>,
    watcher_lock: Option<Sender<()>>,
}

impl TransactionManager {
    pub fn new() -> TransactionManager {
        return TransactionManager {
            inner: TransactionManagerInner::new(),
            watcher_lock: None,
        };
    }

    pub fn set_timeout(&mut self, timeout_ms: i64, poll_interval_ms: u64) {
        let inner_ref = self.inner.clone();
        let (sx, rx) = channel();
        self.watcher_lock = Some(sx);
        thread::spawn(move || {
            loop {
                // Still connected?
                match rx.try_recv() {
                    Ok(_) => {}
                    Err(e) => {
                        match e {
                            TryRecvError::Empty => {}
                            TryRecvError::Disconnected => {
                                break; // Parent went away
                            }
                        }
                    }
                }

                // Perform timeout scan
                match inner_ref.lock() {
                    Ok(mut inner) => {
                        inner.check_expired_transactions(timeout_ms);
                    }
                    Err(e) => {
                        break; // Arc failure
                    }
                }

                thread::sleep(Duration::from_millis(poll_interval_ms));
            }
        });
    }

    pub fn resolve(
        &self,
        transaction_id: &str,
        result: Result<(), ExternalError>,
    ) -> Result<(), RelayError> {
        match self.inner.lock() {
            Ok(mut inner) => {
                inner.resolve_pending(transaction_id, result.map_err(|e| RelayError::ExternalError(e)));
                Ok(())
            }
            Err(e) => Err(RelayError::ArcMutexFailure),
        }
    }

    pub fn defer(&self, transaction_id: &str) -> impl Future<Item=(), Error=RelayError> {
        match self.inner.lock() {
            Ok(mut inner) => {
                let (sx, rx) = oneshot::channel();
                inner.save_pending_transaction(transaction_id, sx);
                Either::A(rx.then(|r| match r {
                    Ok(result) => match result {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e),
                    },
                    Err(e) => Err(RelayError::SyncError(e.description().to_string())),
                }))
            }
            Err(e) => Either::B(futures::failed(RelayError::ArcMutexFailure)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::testing::block_on_future;
    use crate::infrastructure::transaction_manager::TransactionManager;
    use futures::Future;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_create_transaction_manager() {
        let manager = TransactionManager::new();
        let _ = manager.clone();
    }

    #[test]
    fn test_create_and_resolve_transaction() {
        let public_manager = TransactionManager::new();
        let backend_manager = public_manager.clone();

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            backend_manager.resolve("transaction1234", Ok(())).unwrap();
        });

        let promised_result = public_manager.defer("transaction1234");
        let result = block_on_future(promised_result);
        assert!(result.is_ok());
    }
}
