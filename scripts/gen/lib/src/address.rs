//! Address and WIF encoders for the Bitcoin mainnet.

use bitcoin::key::UntweakedPublicKey;
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::{Address, CompressedPublicKey, Network, PrivateKey, PublicKey, XOnlyPublicKey};

use crate::error::{GenError, Result};

/// The mainnet address encodings used by the generation tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressScheme {
    /// Legacy `1...` pay-to-public-key-hash (BIP44).
    P2pkh,
    /// `3...` P2SH-wrapped P2WPKH (BIP49).
    P2shP2wpkh,
    /// `bc1q...` native SegWit pay-to-witness-public-key-hash (BIP84).
    P2wpkh,
    /// `bc1p...` Taproot output (BIP86).
    P2tr,
}

impl AddressScheme {
    /// Resolves the textual name used in the JSON output.
    ///
    /// Returns `None` for unknown names.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "p2pkh" => Some(Self::P2pkh),
            "p2sh-p2wpkh" => Some(Self::P2shP2wpkh),
            "p2wpkh" => Some(Self::P2wpkh),
            "p2tr" => Some(Self::P2tr),
            _ => None,
        }
    }

    /// The textual name used in the JSON output.
    pub const fn name(self) -> &'static str {
        match self {
            Self::P2pkh => "p2pkh",
            Self::P2shP2wpkh => "p2sh-p2wpkh",
            Self::P2wpkh => "p2wpkh",
            Self::P2tr => "p2tr",
        }
    }

    /// Encodes `public_key` into a mainnet address for this scheme.
    ///
    /// # Errors
    /// Returns [`GenError::Address`] when the key cannot be compressed or the
    /// address cannot be built.
    pub fn encode(self, public_key: &PublicKey) -> Result<Address> {
        let network = Network::Bitcoin;
        let address = match self {
            Self::P2pkh => Address::p2pkh(public_key, network),
            Self::P2wpkh => Address::p2wpkh(&compressed(public_key)?, network),
            Self::P2shP2wpkh => Address::p2shwpkh(&compressed(public_key)?, network),
            Self::P2tr => {
                let untweaked = UntweakedPublicKey::from(XOnlyPublicKey::from(*public_key));
                Address::p2tr(&Secp256k1::new(), untweaked, None, network)
            }
        };
        Ok(address)
    }
}

/// Encodes `secret` as a compressed WIF string (starts with `K` or `L`).
///
/// # Errors
/// Never fails today; the result type keeps the encoder API uniform with
/// [`AddressScheme::encode`].
pub fn wif_compressed(secret: &SecretKey) -> Result<String> {
    Ok(to_wif(*secret, true))
}

/// Encodes `secret` as an uncompressed WIF string (starts with `5`).
///
/// # Errors
/// Never fails today; the result type keeps the encoder API uniform with
/// [`AddressScheme::encode`].
pub fn wif_uncompressed(secret: &SecretKey) -> Result<String> {
    Ok(to_wif(*secret, false))
}

fn to_wif(secret: SecretKey, compressed: bool) -> String {
    let private = PrivateKey {
        compressed,
        network: Network::Bitcoin.into(),
        inner: secret,
    };
    private.to_wif()
}

/// Encodes `public_key` as a mainnet P2PKH address, honouring the key's
/// compression flag (brain wallets use uncompressed keys).
///
/// # Errors
/// Propagates any failure from [`AddressScheme::encode`].
pub fn p2pkh_address(public_key: &PublicKey) -> Result<Address> {
    AddressScheme::P2pkh.encode(public_key)
}

fn compressed(public_key: &PublicKey) -> Result<CompressedPublicKey> {
    CompressedPublicKey::try_from(*public_key)
        .map_err(|_| GenError::InvalidKey("public key cannot be compressed".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compressed public key of the private key 1.
    fn one_key() -> PublicKey {
        let mut secret = [0u8; 32];
        secret[31] = 1;
        let secret = SecretKey::from_slice(&secret).expect("valid secret");
        PublicKey::new(secret.public_key(&Secp256k1::new()))
    }

    #[test]
    fn encodes_a_known_p2pkh_address() {
        let address = AddressScheme::P2pkh.encode(&one_key()).expect("encodes");

        // BIP44 test vector, private key 1.
        assert_eq!(address.to_string(), "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH");
    }

    #[test]
    fn encodes_a_known_p2sh_p2wpkh_address() {
        let address = AddressScheme::P2shP2wpkh
            .encode(&one_key())
            .expect("encodes");

        // BIP49 test vector, private key 1.
        assert_eq!(address.to_string(), "3JvL6Ymt8MVWiCNHC7oWU6nLeHNJKLZGLN");
    }

    #[test]
    fn encodes_a_known_p2wpkh_address() {
        let address = AddressScheme::P2wpkh.encode(&one_key()).expect("encodes");

        // BIP84 test vector, private key 1.
        assert_eq!(
            address.to_string(),
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
        );
    }

    #[test]
    fn encodes_a_known_p2tr_address() {
        let address = AddressScheme::P2tr.encode(&one_key()).expect("encodes");

        // BIP86 test vector, private key 1.
        assert_eq!(
            address.to_string(),
            "bc1pmfr3p9j00pfxjh0zmgp99y8zftmd3s5pmedqhyptwy6lm87hf5sspknck9"
        );
    }

    #[test]
    fn encodes_compressed_and_uncompressed_wif() {
        let mut secret = [0u8; 32];
        secret[31] = 1;
        let secret = SecretKey::from_slice(&secret).expect("valid secret");

        assert!(wif_compressed(&secret).expect("encodes").starts_with('K'));
        assert!(wif_uncompressed(&secret).expect("encodes").starts_with('5'));
    }

    #[test]
    fn unknown_scheme_names_are_rejected() {
        assert!(AddressScheme::from_name("p2wpkh").is_some());
        assert!(AddressScheme::from_name("p2wsh").is_none());
    }
}
