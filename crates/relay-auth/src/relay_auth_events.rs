use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
pub enum RelayAuthEvent {
    /// Authorize this socket connection
    Authorize(Token)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
pub struct Token {
    /// When the token was created
    timestamp: i64,

    /// The set of requested claims
    claims: Vec<Claim>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
pub struct Claim {
    pub claim: String,
    pub token: String,
}