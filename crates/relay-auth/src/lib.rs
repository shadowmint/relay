pub(crate) mod errors;
pub(crate) mod auth_secret_provider;
pub(crate) mod events;
pub(crate) mod infrastructure;
pub(crate) mod auth_provider;
pub(crate) mod auth_provider_config;

pub use auth_provider_config::AuthProviderConfig;
pub use auth_provider::AuthProvider;
pub use auth_provider::AuthResponse;

pub use auth_secret_provider::AuthSecretProvider;

pub use errors::AuthError;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
