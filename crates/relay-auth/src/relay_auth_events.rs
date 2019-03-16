use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
pub enum RelayAuthEvent {
    /// Authorize this socket connection
    Authorize(Claims)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
pub struct Claims {
    /// When the token was created
    pub timestamp: i64,

    /// The set of requested claims
    pub claims: Vec<Claim>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
pub struct Claim {
    /// The public claim for this user
    pub claim: String,

    /// This token should be a hash of format!("{}:{}:{}", timestamp, claim, claim private key)
    pub token: String,
}

pub struct VerifiedClaims {
    /// When the token was created
    pub timestamp: i64,

    /// When the token expires
    pub expires: i64,

    /// The set of requested claims
    pub claims: Vec<VerifiedClaim>,
}

pub struct VerifiedClaim {
    /// The public claim for this user
    pub claim: String,

    /// Is this a valid token?
    pub valid: bool,
}