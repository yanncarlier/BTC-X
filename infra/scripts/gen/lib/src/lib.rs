//! Shared building blocks for the `btcx` generation tools.
//!
//! Every binary in `gen/` is a thin wrapper around this crate: it names the
//! derivation programme it implements (see [`spec`]) and prints the JSON report
//! the matching Python script produces.

pub mod address;
pub mod brain_wallet;
pub mod cli;
pub mod error;
pub mod json;
pub mod keys;
pub mod mnemonic;
pub mod report;
pub mod spec;
pub mod template;

pub use address::{wif_compressed, wif_uncompressed, AddressScheme};
pub use error::{GenError, Result};
pub use keys::{ExtendedKey, KeyVersion, PathElement};
pub use report::{
    AddressRecord, BrainWalletReport, DerivationReport, GeneratedMnemonicReport, TemplateAddress,
    TemplateGroup,
};
pub use spec::{DerivationSpec, IndexStyle};
