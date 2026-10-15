//! Error types for the Bitcoin Tools library

use bitcoin::{address, bip32, psbt};
use std::fmt;

/// Common error type for the library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Bitcoin-specific errors
    #[error("Bitcoin error: {0}")]
    Bitcoin(#[from] bitcoin::consensus::encode::Error),

    /// Hex encoding/decoding errors
    #[error("Hex decode error: {0}")]
    Hex(#[from] bitcoin::hashes::hex::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Network-related errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Invalid network specification
    #[error("Invalid network: {0}")]
    InvalidNetwork(String),

    /// Invalid Bitcoin address
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Invalid transaction data
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    /// Transaction signing errors
    #[error("Transaction signing failed: {0}")]
    SigningError(String),

    /// Insufficient funds for transaction
    #[error("Insufficient funds")]
    InsufficientFunds,

    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Custom error message
    #[error("{0}")]
    Custom(String),

    /// PSBT (Partially Signed Bitcoin Transaction) errors
    #[error("PSBT error: {0}")]
    PsbtError(String),

    /// Script-related errors
    #[error("Script error: {0}")]
    ScriptError(String),

    /// Descriptor parsing errors
    #[error("Descriptor error: {0}")]
    DescriptorError(String),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Type alias for Result<T, Error>
pub type Result<T> = std::result::Result<T, Error>;

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Custom(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Custom(s.to_string())
    }
}

impl From<address::Error> for Error {
    fn from(e: address::Error) -> Self {
        Error::InvalidAddress(e.to_string())
    }
}

impl From<bip32::Error> for Error {
    fn from(e: bip32::Error) -> Self {
        Error::Custom(format!("BIP32 error: {}", e))
    }
}

impl From<psbt::Error> for Error {
    fn from(e: psbt::Error) -> Self {
        Error::PsbtError(e.to_string())
    }
}

impl From<bitcoin::blockdata::script::Error> for Error {
    fn from(e: bitcoin::blockdata::script::Error) -> Self {
        Error::ScriptError(e.to_string())
    }
}

impl From<bitcoin::secp256k1::Error> for Error {
    fn from(e: bitcoin::secp256k1::Error) -> Self {
        Error::SigningError(e.to_string())
    }
}

/// Helper trait to add context to errors
pub trait Context<T, E> {
    /// Add context to an error
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static;
}

impl<T, E> Context<T, E> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| Error::Custom(format!("{}: {}", context, e)))
    }
}
