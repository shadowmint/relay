use crate::errors::relay_error::RelayError;

pub trait RelaySerializer<T> {
    fn serialize(&self, event: &T) -> Result<String, RelayError>;
    fn deserialize(&self, raw: &str) -> Result<T, RelayError>;
}
