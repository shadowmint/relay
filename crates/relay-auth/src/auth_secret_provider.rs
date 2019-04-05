/// Returns secrets for the auth layer to use.
pub trait AuthSecretProvider {
    /// Return the secret for a given key, or None.
    /// If the provider needs to be mutable it should maintain its own Arc.
    fn secret_for(&self, key: &str) -> Option<String>;
}