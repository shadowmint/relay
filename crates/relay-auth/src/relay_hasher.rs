use crate::Claims;
use crate::relay_auth_events::VerifiedClaims;
use crate::RelayAuthConfig;
use crate::Claim;
use crate::relay_auth_events::VerifiedClaim;

pub struct RelayHasher {
    config: RelayAuthConfig
}

impl RelayHasher {
    pub fn new(config: RelayAuthConfig) -> RelayHasher {
        RelayHasher {
            config
        }
    }

    pub fn apply(&self, claims: Claims) -> Claims {
        claims
    }

    pub fn verify(&self, claims: Claims) -> VerifiedClaims {
        VerifiedClaims {
            timestamp: claims.timestamp,
            expires: claims.timestamp + self.config.token_expires_in,
            claims: claims.claims.iter().map(|i| self.validate_claim(i)).collect(),
        }
    }

    fn validate_claim(&self, claim: &Claim) -> VerifiedClaim {
        VerifiedClaim {
            claim: claim.claim.to_string(),
            valid: false, // TODO: Fix this
        }
    }
}