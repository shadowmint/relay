use crate::errors::relay_error::RelayError;
use crate::infrastructure::auth_helper::AuthHelper;
use crate::infrastructure::backend::{Backend, BackendOptions};
use crate::infrastructure::relay_event::RelayEvent;
use crate::infrastructure::transaction_manager::TransactionManager;
use crate::ClientOptions;
use futures::future::Either;
use futures::Future;
use relay_core::events::client_event::ClientExternalEvent;
use relay_core::events::master_event::MasterExternalEvent;
use relay_core::model::client_metadata::ClientMetadata;
use relay_core::model::master_metadata::MasterMetadata;
use uuid::Uuid;

pub struct Client {
    connection: Backend,
}

impl Client {
    pub fn new(options: ClientOptions) -> impl Future<Item = Client, Error = RelayError> {
        let opt_a = options.clone();
        let opt_b = options.clone();
        let opt_c = options.clone();
        Backend::new(BackendOptions {
            remote: options.remote.clone(),
            target: options.backend,
            transaction_manager: TransactionManager::new(),
        })
        .then(move |b| match b {
            Ok(connection) => match AuthHelper::generate_auth(&opt_a.auth) {
                Ok(signed_auth_request) => {
                    let promise = connection.send(signed_auth_request);
                    Either::B(promise.then(move |r| match r {
                        Ok(_) => Ok(connection),
                        Err(e) => Err(e),
                    }))
                }
                Err(e) => Either::A(futures::failed(e)),
            },
            Err(e) => Either::A(futures::failed(e)),
        })
        .then(move |b| match b {
            Ok(connection) => {
                let promise = connection.send(RelayEvent::Client(ClientExternalEvent::InitializeClient {
                    transaction_id: Uuid::new_v4().to_string(),
                    metadata: ClientMetadata { name: opt_b.client_id },
                }));
                Either::B(promise.then(move |r| match r {
                    Ok(_) => Ok(connection),
                    Err(e) => Err(e),
                }))
            }
            Err(e) => Either::A(futures::failed(e)),
        })
        .then(move |b| match b {
            Ok(connection) => {
                let promise = connection.send(RelayEvent::Client(ClientExternalEvent::Join {
                    transaction_id: Uuid::new_v4().to_string(),
                    session_id: opt_c.session_id.clone(),
                }));
                Either::B(promise.then(move |r| match r {
                    Ok(_) => Ok(connection),
                    Err(e) => Err(e),
                }))
            }
            Err(e) => Either::A(futures::failed(e)),
        })
        .then(|c| match c {
            Ok(connection) => Ok(Client { connection }),
            Err(e) => Err(e),
        })
    }

    pub fn channel(&self) -> crossbeam::Receiver<RelayEvent> {
        self.connection.channel()
    }

    pub fn send(&self, event: ClientExternalEvent) -> impl Future<Item = (), Error = RelayError> {
        self.connection.send(RelayEvent::Client(event))
    }
}

#[cfg(test)]
mod tests {
    use crate::client::ClientOptions;
    use crate::infrastructure::testing::block_on_future;
    use crate::{AuthOptions, BackendType, Client};

    #[test]
    fn test_create_master() {
        let _ = block_on_future(Client::new(ClientOptions {
            client_id: "Client A".to_string(),
            session_id: "Master".to_string(),
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
