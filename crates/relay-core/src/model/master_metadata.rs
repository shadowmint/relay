use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MasterMetadata {
    /// The name of this master
    pub master_id: String,

    /// The maximum clients count to allow
    pub max_clients: u32,
}
