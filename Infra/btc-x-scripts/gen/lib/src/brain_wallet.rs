//! Brain wallet key derivation.
//!
//! # Warning
//! Brain wallets derive keys straight from a passphrase. Anyone who guesses the
//! passphrase owns the coins, so this exists for education and research only.

use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::PublicKey;
use sha2::{Digest, Sha256};

use crate::address::wif_uncompressed;
use crate::error::{GenError, Result};
use crate::report::BrainWalletReport;

/// Warning attached to every brain wallet report.
pub const WARNING: &str = "Brain wallet (WARNING: Highly insecure, for educational purposes only)";

/// Derives a brain wallet from `passphrase`.
///
/// The private key is the SHA-256 digest of the passphrase; the address is the
/// P2PKH address of the matching *uncompressed* public key and the WIF is
/// therefore an uncompressed key (it starts with `5`).
///
/// # Errors
/// Returns [`GenError::InvalidKey`] when the digest is not a valid secp256k1
/// secret, which only happens for a vanishingly small fraction of passphrases.
pub fn derive(passphrase: &str) -> Result<BrainWalletReport> {
    let digest = Sha256::digest(passphrase.as_bytes());
    let secret = SecretKey::from_slice(&digest).map_err(|e| GenError::InvalidKey(e.to_string()))?;

    let secp = Secp256k1::new();
    let public = PublicKey::new_uncompressed(secret.public_key(&secp));
    let uncompressed = public.inner.serialize_uncompressed();

    Ok(BrainWalletReport {
        description: WARNING,
        address: crate::address::p2pkh_address(&public)?.to_string(),
        public_key: hex::encode(uncompressed),
        private_key: hex::encode(digest),
        wif: wif_uncompressed(&secret)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_reference_wallet_for_the_passphrase_testpass() {
        let wallet = derive("testpass").expect("derives");

        assert_eq!(wallet.address, "1BQt63DNaneCnDqX8iJt2YyLmouZcs1wx9");
        assert_eq!(
            wallet.wif,
            "5Hy1tnNFwueXt52s5rkDPHacgmybDsvoQ7T3gAhrQY1irmx89dJ"
        );
        assert_eq!(
            wallet.private_key,
            "13d249f2cb4127b40cfa757866850278793f814ded3c587fe5889e889a7a9f6c"
        );
        assert!(wallet.public_key.starts_with("04"));
        assert_eq!(wallet.public_key.len(), 130);
    }
}
