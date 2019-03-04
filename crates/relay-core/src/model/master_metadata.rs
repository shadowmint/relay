use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MasterMetadata {
    /// The name of this master
    pub master_id: String,

    /// The maximum clients count to allow
    pub max_clients: u32,
}