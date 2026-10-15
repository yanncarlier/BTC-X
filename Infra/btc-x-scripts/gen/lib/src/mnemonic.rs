//! BIP39 mnemonic helpers.

use bip39::{Language, Mnemonic};

use crate::error::{GenError, Result};
use crate::keys::{ExtendedKey, KeyVersion};
use crate::report::GeneratedMnemonicReport;

/// The well known public test mnemonic used when the caller supplies none.
///
/// # Warning
/// This phrase is public knowledge and must never hold real funds.
pub const DEFAULT_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// The English BIP39 wordlist.
pub const LANGUAGE: Language = Language::English;

/// Validates `mnemonic` against the BIP39 checksum and returns the parsed value.
///
/// # Errors
/// Returns [`GenError::InvalidMnemonic`] when the word count is unsupported, a
/// word is unknown, or the checksum does not match.
pub fn parse(mnemonic: &str) -> Result<Mnemonic> {
    Mnemonic::parse_in(LANGUAGE, mnemonic.trim()).map_err(|_| GenError::InvalidMnemonic)
}

/// Derives the 64-byte BIP39 seed for `mnemonic` and the optional `passphrase`.
///
/// # Errors
/// Returns [`GenError::InvalidMnemonic`] when the phrase fails validation.
pub fn seed(mnemonic: &str, passphrase: &str) -> Result<[u8; 64]> {
    Ok(parse(mnemonic)?.to_seed(passphrase))
}

/// Generates a fresh English mnemonic of `strength_bits` bits.
///
/// Valid strengths are 128, 160, 192, 224 and 256, which map to 12, 15, 18, 21
/// and 24 words respectively.
///
/// # Errors
/// Returns [`GenError::Randomness`] when the entropy source fails and
/// [`GenError::Usage`] when `strength_bits` is not a valid strength.
pub fn generate(strength_bits: usize) -> Result<Mnemonic> {
    let word_count = match strength_bits {
        128 => 12,
        160 => 15,
        192 => 18,
        224 => 21,
        256 => 24,
        other => {
            return Err(GenError::Usage(format!(
                "strength must be one of 128, 160, 192, 224 or 256 (got {other})"
            )));
        }
    };

    Mnemonic::generate_in(LANGUAGE, word_count).map_err(|e| GenError::Randomness(e.to_string()))
}

/// Generates a mnemonic and the seed and BIP32 root key derived from it.
///
/// The Python original always derives the seed with an empty passphrase.
///
/// # Errors
/// Returns an error when `strength_bits` is invalid, randomness fails, or the
/// resulting seed cannot become a BIP32 master key.
pub fn generate_report(strength_bits: usize) -> Result<GeneratedMnemonicReport> {
    let mnemonic = generate(strength_bits)?;
    let seed = mnemonic.to_seed("");
    let master = ExtendedKey::master(&seed)?;

    Ok(GeneratedMnemonicReport {
        bip39_mnemonic: mnemonic.to_string(),
        bip39_seed: hex::encode(seed),
        bip32_root_key: master.to_extended_private(KeyVersion::Xpub),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_default_test_mnemonic() {
        let mnemonic = parse(DEFAULT_MNEMONIC).expect("valid mnemonic");

        assert_eq!(mnemonic.word_count(), 12);
    }

    #[test]
    fn rejects_a_mnemonic_with_a_broken_checksum() {
        let broken = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";

        assert!(matches!(parse(broken), Err(GenError::InvalidMnemonic)));
    }

    #[test]
    fn seed_matches_bip39_reference_vector() {
        let seed = seed(DEFAULT_MNEMONIC, "").expect("valid mnemonic");

        assert_eq!(hex::encode(seed), "5eb00bbddcf069084889a8ab9155568165f5c453ccb85e70811aaed6f6da5fc19a5ac40b389cd370d086206dec8aa6c43daea6690f20ad3d8d48b2d2ce9e38e4");
    }

    #[test]
    fn generates_phrases_of_the_requested_length() {
        let mnemonic = generate(256).expect("generates");

        assert_eq!(mnemonic.word_count(), 24);
    }

    #[test]
    fn rejects_unsupported_strengths() {
        assert!(generate(129).is_err());
    }
}
