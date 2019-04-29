use crate::infrastructure::backend::BackendType;

#[derive(Clone)]
pub struct MasterOptions {
    pub remote: String,
    pub backend: BackendType,
    pub master_id: String,
    pub max_clients: u32,
    pub auth: AuthOptions,
}

#[derive(Clone)]
pub struct ClientOptions {
    pub remote: String,
    pub backend: BackendType,
    pub client_id: String,
    pub session_id: String,
    pub auth: AuthOptions,
}

#[derive(Clone)]
pub struct AuthOptions {
    pub session_expires_secs: i64,
    pub key: String,
    pub secret: String,
}
