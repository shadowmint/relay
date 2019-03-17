use crate::Claims;
use crate::relay_auth_events::VerifiedClaims;
use crate::RelayAuthConfig;
use crate::Claim;
use crate::relay_auth_events::VerifiedClaim;
use crate::RelayAuthError;

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

    fn verify_config(&self) -> Result<(), RelayAuthError> {
        Ok(())
    }

    fn encrypt_token(&self, token: &str) -> String {
        return format!("N/A");
    }

    fn descrypt_token(&self, token: &str) -> String {
        return format!("Nope");
    }
}


#[cfg(test)]
mod tests {
    use openssl::rsa::Rsa;

    const PRIVATE_KEY: &'static str = "-----BEGIN RSA PRIVATE KEY-----
Proc-Type: 4,ENCRYPTED
DEK-Info: AES-128-CBC,EB145ADF1285978D342229074F3EB8F1

Da4PTkg5h4i/+mMCuW71IYw11o/nPaqsuiIZkNAmYlP6Hn5vMaFSaVmspK9ajezP
KkJxCi3yRqWLuqMTW8KRGSXi8ujCyo/ERiHRg2U4nga0Rgdz6Wo1vHrGbG06EVdI
cfK5Qt2nJAE29UrcjUeKVQ+M5E9IbGsY5DPix+6/EJ2nvkyYGoB/XusxmIYWYqou
3pDdDWDyo0FsZG8ZjEjR8pmkfPMf1+duIblfkfQC8LBZDi1sMYPhqrcColhvc1jy
6xu8Dsb4ZrEsbYT+7w5+TGr492dGbXO4sfxinkSUDhRq90VI6UxcJ73Cd1xoJIHh
37ZQbDCJE7Mfq9RxJoM4cFtdr7F1HhwSVpJZ3Ro9/vdlJP+NYKXGEL/EWqOWf04k
20ZMwtCbGtZ6GD7D1RfhIcXgLDjSVw2TKi2nVNhUBG90CmfhB/B0UvG5Nb550zB+
CuFiuh2BmavhikUzS4RBWrSjdjZz7TsLsSY6G1cVnR1uQ1k9Kc2KmCpH5+22TQHU
VN48dD+zDdpZWnKCcQh4EJH2Nm5uNijlIGgMkU/e7h0EBohZytbQdLr+DYqfvq85
VqkDlqyMnC+zBSIuynrcM7zJeZRzWro3USLS0YzmyVaFAeVXmxMX793qSzY+c8x/
v2a+6KvQdvHowGJ2cAtO+S8PPuc88uPxW3RdCIv+PWe1y18hyA2GlQynpwakZD2i
PaS1L1dyHd9xPoQNwzN1EDLD5i69tg7bFLOHuIA/VIcWQ+1l8GQKNG+sQngLPHYt
K4xd7V4H8tx21MLAU1DvoTbWw0KfI0au/ibXehiMKhTS9TDaFD7ZZiAwSs8JHcKG
3zwAItUb/WL1yyVIfRE1o2+A3bSYW6vCEqypo6D/VAJGxgkLWFG2ePFt7NZJ4l9z
wuwflb8j62dh4CoXdr3xUnMzwcCLJOb5fxRkJv+/plI9HDkM5n52u1Z95SCnueJy
qneL+hqUM8Tko00i65YslJP02NEI8ZF1aEanL8mpEHC35inj7qdr1nL9Zq2nXno7
bhMz30ePoXmBrJiheXsZXrTvX5h06Tc+IU+P28Bz7qIuW8HJbvrFt4dlSVlyFUcP
epBu1PnyvcRLgdYlhfrID5cx9cU/YiI10ZEGBeesNfTc4p+25XeBfnXf664hgIz5
n+5RaGR0yuPuggaXiUaJ7yVY3e+sLg6uxNWR1ExCArSItlLFsSdPhjL8aARV9xVA
4U1BV17kgWC/yqTyNaGp/AexTDoZXN0aQYr0/PzVAzaN37w6qJzvFOryXRHnF3ab
JrybkXYAjG9Zzy6yP6Rxx4BUMAmdEpZfGNGlRke9pjUWhMFW+pwiel3o/bUPhKJr
Q1mYXju0GHKLxRF+F5uebqLtNKJP0R7X19AFubXgwH3yq0dkDJOkf1mrUMk1T+8U
tU34tGvZTIkvRREaZF0jO93uc5nYaZUljTPyrU+09XQIbHAxL3DNQN0eq8R/zV9g
ZRI/HFn1F2OmOYNxoe6D77jJKDOwv2tl+CMzmVJnn/IAFedv2WckwJY1Yi4cbFMM
FXVXo9Nqh6sMTI/GA/SBsCzbHs+D4is1q5iadurW5uNMZbJx2/hGAKUO6lmbJs6Y
-----END RSA PRIVATE KEY-----";

    const PUBLIC_KEY: &'static str = "-----BEGIN RSA PUBLIC KEY-----
MIIBCgKCAQEAsG8BJ19GhFNdvow3tc3baj5ka8CzYBb1760g3eWT6vN1sGSV5E3g
0/djIfE4oOXeRLJmar7Rm9zRb/M+zqueiLugdjNUkPWb7ZFvI2ZQTeD+Tvb9UAy8
agzemkOuuqQa4Rb7ZuYgTD3KX+AepSHO8xBpaPXeMqTvBYWEpqWvh82m5GuqAtLT
WLc3JBZAXZ3vyWNIzwfiwEet59oxSQuLj7iP6+3MdGXBsLhfypwANadaeTr85X6y
84ZcjeqnJFRWp928VxYYko2OXaHLLwBgOygygxI2lyCrDdx/YI7XbgRvSMweAYTq
4Lqk2xbmUmQJuWTtQdM5LX157QQrHsdzywIDAQAB
-----END RSA PUBLIC KEY-----";

    const PASSPHRASE: &'static str = "test123";

    #[test]
    fn test_parse_public_key() {
        let _ = Rsa::public_key_from_pem_pkcs1(PUBLIC_KEY.as_bytes()).unwrap();
    }

    #[test]
    fn test_parse_private_key() {
        let _ = Rsa::private_key_from_pem_passphrase(PRIVATE_KEY.as_bytes(), PASSPHRASE.as_bytes()).unwrap();
    }

    /*
    pub fn private_decrypt(
    &self,
    from: &[u8],
    to: &mut [u8],
    padding: Padding
) -> Result<usize, ErrorStack>
[src][−]
Decrypts data using the private key, returning the number of decrypted bytes.

Panics
Panics if self has no private components, or if to is smaller than self.size().

pub fn private_encrypt(
    &self,
    from: &[u8],
    to: &mut [u8],
    padding: Padding
) -> Result<usize, ErrorStack>
[src][−]
Encrypts data using the private key, returning the number of encrypted bytes.
*/
}
