mod relay_auth;
mod relay_auth_config;
mod relay_auth_error;
mod relay_auth_events;
pub(crate) mod relay_hasher;

pub use relay_auth::RelayAuth;
pub use relay_auth_config::RelayAuthConfig;
pub use relay_auth_error::RelayAuthError;

pub use relay_auth_events::Claim;
pub use relay_auth_events::Claims;
pub use relay_auth_events::RelayAuthEvent;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
