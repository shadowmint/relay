use crate::errors::relay_error::RelayError;
use crate::infrastructure::auth_helper::AuthHelper;
use crate::infrastructure::backend::{Backend, BackendOptions};
use crate::infrastructure::relay_event::RelayEvent;
use crate::infrastructure::transaction_manager::TransactionManager;
use crate::MasterOptions;

use relay_core::events::master_event::MasterExternalEvent;
use relay_core::model::master_metadata::MasterMetadata;
use std::future::Future;
use uuid::Uuid;

pub struct Master {
    connection: Backend,
}

impl Master {
    pub async fn new(options: MasterOptions) -> Result<Master, RelayError> {
        let backend = Backend::new(BackendOptions {
            auth: AuthHelper::generate_auth(&options.auth),
            remote: options.remote.clone(),
            target: options.backend,
            transaction_manager: TransactionManager::new(),
        })
        .await?;

        backend
            .send(RelayEvent::Master(MasterExternalEvent::InitializeMaster {
                transaction_id: Uuid::new_v4().to_string(),
                metadata: MasterMetadata {
                    max_clients: options.max_clients,
                    master_id: options.master_id,
                },
            }))
            .await?;

        Ok(Master { connection: backend })
    }

    pub fn channel(&self) -> crossbeam::Receiver<RelayEvent> {
        self.connection.channel()
    }

    pub fn send(&self, event: MasterExternalEvent) -> impl Future<Output = Result<(), RelayError>> + '_ {
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
