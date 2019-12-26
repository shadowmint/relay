use crate::errors::relay_error::RelayError;
use crate::infrastructure::auth_helper::AuthHelper;
use crate::infrastructure::backend::{Backend, BackendOptions};
use crate::infrastructure::relay_event::RelayEvent;
use crate::infrastructure::transaction_manager::TransactionManager;
use crate::ClientOptions;
use relay_core::events::client_event::ClientExternalEvent;

use relay_core::model::client_metadata::ClientMetadata;

use std::future::Future;
use uuid::Uuid;

pub struct Client {
    connection: Backend,
}

impl Client {
    pub async fn new(options: ClientOptions) -> Result<Client, RelayError> {
        let client = Backend::new(BackendOptions {
            auth: AuthHelper::generate_auth(&options.auth),
            remote: options.remote.clone(),
            target: options.backend,
            transaction_manager: TransactionManager::new(),
        })
        .await?;

        client
            .send(RelayEvent::Client(ClientExternalEvent::InitializeClient {
                transaction_id: Uuid::new_v4().to_string(),
                metadata: ClientMetadata { name: options.client_id },
            }))
            .await?;

        client
            .send(RelayEvent::Client(ClientExternalEvent::Join {
                transaction_id: Uuid::new_v4().to_string(),
                session_id: options.session_id.clone(),
            }))
            .await?;

        Ok(Client { connection: client })
    }

    pub fn channel(&self) -> crossbeam::Receiver<RelayEvent> {
        self.connection.channel()
    }

    pub fn send(&self, event: ClientExternalEvent) -> impl Future<Output = Result<(), RelayError>> + '_ {
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
