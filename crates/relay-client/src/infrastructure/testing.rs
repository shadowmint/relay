use crate::infrastructure::managed_connection::{ManagedConnectionHandler};
use futures::IntoFuture;

pub fn block_on_future<F>(f: F) -> Result<F::Item, F::Error>
where
    F: IntoFuture,
    F::Future: Send + 'static,
    F::Item: Send + 'static,
    F::Error: Send + 'static,
{
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(f.into_future())
}