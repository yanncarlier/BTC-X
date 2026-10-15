//! Error types shared by the `btcx_gen` generation tools.

use thiserror::Error;

/// Errors that can occur while validating inputs or deriving keys/addresses.
#[derive(Debug, Error)]
pub enum GenError {
    /// The supplied phrase is not a valid BIP39 mnemonic.
    #[error("Invalid mnemonic phrase provided. Please check the words and try again.")]
    InvalidMnemonic,

    /// BIP32 derivation failed (for example, an invalid child index).
    #[error("Failed to derive child key: {0}")]
    Derivation(String),

    /// A derived secret or public key was rejected by the secp256k1 backend.
    #[error("Invalid key material: {0}")]
    InvalidKey(String),

    /// An address could not be encoded for the requested scheme.
    #[error("Failed to encode address: {0}")]
    Address(String),

    /// The command line could not be interpreted.
    #[error("{0}")]
    Usage(String),

    /// Random number generation failed.
    #[error("Failed to generate randomness: {0}")]
    Randomness(String),
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, GenError>;
