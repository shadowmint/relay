use futures::sync::oneshot::Sender;
use relay_core::model::external_error::{ExternalError, ErrorCode};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::Utc;
use crate::errors::relay_error::RelayError;

struct PendingTransaction {
    promise: Sender<Result<(), RelayError>>,
    started: i64,
}

pub struct TransactionManagerInner {
    pending: HashMap<String, PendingTransaction>,
}

impl TransactionManagerInner {
    pub fn new() -> Arc<Mutex<TransactionManagerInner>> {
        return Arc::new(Mutex::new(TransactionManagerInner {
            pending: HashMap::new(),
        }));
    }

    pub fn check_expired_transactions(&mut self, timeout_ms: i64) {
        let threshold = Utc::now().timestamp_millis() - timeout_ms;
        let expired: Vec<String> = self.pending
            .iter()
            .filter(|(_, v)| {
                v.started < threshold
            })
            .map(|(k, _)| k.to_string())
            .collect();
        for expired_key in expired {
            self.resolve_pending(&expired_key, Err(RelayError::TransactionExpired));
        }
    }

    pub fn resolve_pending(&mut self, transaction_id: &str, result: Result<(), RelayError>) {
        match self.pending.remove(transaction_id) {
            Some(t) => {
                let _ = t.promise.send(result);
            }
            None => {}
        }
    }

    pub fn save_pending_transaction(
        &mut self,
        transaction_id: &str,
        promise: Sender<Result<(), RelayError>>,
    ) {
        self.pending.insert(transaction_id.to_string(), PendingTransaction {
            promise,
            started: Utc::now().timestamp_millis(),
        });
    }
}
