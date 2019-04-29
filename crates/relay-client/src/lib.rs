pub(crate) mod errors;

pub(crate) mod client;
pub(crate) mod client_typed;

pub(crate) mod master;
pub(crate) mod master_typed;

pub(crate) mod infrastructure;
pub(crate) mod options;

pub use errors::relay_error::RelayError;

pub use options::AuthOptions;
pub use options::ClientOptions;
pub use options::MasterOptions;

pub use master::Master;
pub use master_typed::MasterEvent;
pub use master_typed::MasterTyped;

pub use client::Client;
pub use client_typed::ClientEvent;
pub use client_typed::ClientTyped;

// For testing
pub use infrastructure::backend::BackendType;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
