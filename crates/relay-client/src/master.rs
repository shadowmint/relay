use crate::errors::relay_error::RelayError;
use crate::infrastructure::auth_helper::AuthHelper;
use crate::infrastructure::backend::{Backend, BackendOptions};
use crate::infrastructure::relay_event::RelayEvent;
use crate::infrastructure::transaction_manager::TransactionManager;
use crate::MasterOptions;
use futures::future::Either;
use futures::Future;
use relay_core::events::master_event::MasterExternalEvent;
use relay_core::model::master_metadata::MasterMetadata;
use uuid::Uuid;

pub struct Master {
    connection: Backend,
}

impl Master {
    pub fn new(options: MasterOptions) -> impl Future<Item = Master, Error = RelayError> {
        let opt_a = options.clone();
        let opt_b = options.clone();
        Backend::new(BackendOptions {
            auth: AuthHelper::generate_auth(&options.auth),
            remote: options.remote.clone(),
            target: options.backend,
            transaction_manager: TransactionManager::new(),
        })
        .then(move |b| match b {
            Ok(connection) => {
                let promise = connection.send(RelayEvent::Master(MasterExternalEvent::InitializeMaster {
                    transaction_id: Uuid::new_v4().to_string(),
                    metadata: MasterMetadata {
                        max_clients: opt_b.max_clients,
                        master_id: opt_b.master_id,
                    },
                }));
                Either::B(promise.then(move |r| match r {
                    Ok(_) => Ok(connection),
                    Err(e) => Err(e),
                }))
            }
            Err(e) => Either::A(futures::failed(e)),
        })
        .then(|c| match c {
            Ok(connection) => Ok(Master { connection }),
            Err(e) => Err(e),
        })
    }

    pub fn channel(&self) -> crossbeam::Receiver<RelayEvent> {
        self.connection.channel()
    }

    pub fn send(&self, event: MasterExternalEvent) -> impl Future<Item = (), Error = RelayError> {
        self.connection.send(RelayEvent::Master(event))
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::backend::BackendType;
    use crate::infrastructure::testing::block_on_future;
    use crate::master::{Master, MasterOptions};
    use crate::AuthOptions;

    #[test]
    fn test_create_master() {
        let _ = block_on_future(Master::new(MasterOptions {
            master_id: "Master".to_string(),
            max_clients: 10,
            remote: "123".to_string(),
            backend: BackendType::Mock,
            auth: AuthOptions {
                key: "1234567890".to_string(),
                secret: "1234567890".to_string(),
                session_expires_secs: 1800,
            },
        }))
        .unwrap();
    }
}
