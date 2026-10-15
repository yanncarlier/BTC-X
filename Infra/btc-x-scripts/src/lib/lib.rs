//! Bitcoin Tools Library
//! Common types and utilities for Bitcoin-related command-line tools.

pub mod error;
pub mod types;

// Re-exports
pub use bitcoin::{
    absolute, consensus, hashes, hashes::hex::FromHex, secp256k1, Address, Amount, Block,
    BlockHash, Network, OutPoint, PublicKey, Script, Sequence, Transaction, TxIn, TxOut, Txid,
    Witness,
};
pub use error::{Error, Result};
pub use types::*;
