use crate::errors::relay_error::RelayError;
use crate::infrastructure::managed_connection::ManagedConnectionHandler;
use crate::infrastructure::relay_event::RelayEvent;
use crate::infrastructure::transaction_manager::TransactionManager;
use futures::future;
use futures::Future;

pub struct MockBackend {
    transaction_manager: TransactionManager,
    auto_resolve: bool,
}

impl MockBackend {
    pub fn new(
        transaction_manager: TransactionManager,
        auto_resolve: bool,
    ) -> impl Future<Output = Result<Box<dyn ManagedConnectionHandler + Send + 'static>, RelayError>> {
        future::ok(Box::new(MockBackend {
            transaction_manager,
            auto_resolve,
        }) as Box<dyn ManagedConnectionHandler + Send + 'static>)
    }
}

impl ManagedConnectionHandler for MockBackend {
    fn send(&self, event: RelayEvent) -> Result<(), ()> {
        let _raw = serde_json::to_string(&event).unwrap();
        let transaction_id = event.transaction_id();
        if !self.auto_resolve {
            return Ok(());
        }
        match transaction_id.as_ref() {
            Some(id) => match self.transaction_manager.resolve(id, Ok(())) {
                Ok(_) => {
                    println!("MOCK: resolved transaction {}", id);

                    Ok(())
                }
                Err(e) => {
                    println!("MOCK: {:?}", e);
                    Err(())
                }
            },
            None => Err(()),
        }
    }
}
