use std::future::Future;

pub fn block_on_future<F: Future>(future: F) -> F::Output {
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(future)
}
