//! JSON payload types emitted by the generation tools.

use serde::Serialize;

/// A single derived address and the keys behind it.
#[derive(Debug, Clone, Serialize)]
pub struct AddressRecord {
    /// Zero based address index, or `null` for templates without an index.
    pub index: Option<u32>,
    /// The BIP32 path this record was derived from.
    pub derivation_path: String,
    /// The encoded mainnet address.
    pub address: String,
    /// The compressed public key, hex encoded.
    pub public_key: String,
    /// The raw 32-byte private key, hex encoded.
    pub private_key: String,
    /// The private key in Wallet Import Format.
    pub wif: String,
}

/// The document produced by each `m-...` derivation tool.
#[derive(Debug, Clone, Serialize)]
pub struct DerivationReport {
    /// The mnemonic the addresses were derived from.
    pub mnemonic_phrase: String,
    /// The optional BIP39 passphrase (empty by default).
    pub passphrase: String,
    /// The BIP39 seed, hex encoded.
    pub seed_hex: String,
    /// Human readable description of the wallet scheme in use.
    pub address_type: String,
    /// The account level extended public key, when the wallet exposes one.
    pub account_extended_public_key: Option<String>,
    /// The account level extended private key, when the wallet exposes one.
    pub account_extended_private_key: Option<String>,
    /// Every derived address, in index order.
    pub addresses: Vec<AddressRecord>,
}

/// One address produced by the `all_address_types` template walk.
///
/// This repeats the scheme on every entry, which the `m-...` tools do not.
#[derive(Debug, Clone, Serialize)]
pub struct TemplateAddress {
    /// Zero based address index, or `null` for templates without an index.
    pub index: Option<u32>,
    /// The BIP32 path this record was derived from.
    pub derivation_path: String,
    /// The address scheme used to encode the key.
    pub address_type: String,
    /// The encoded mainnet address.
    pub address: String,
    /// The compressed public key, hex encoded.
    pub public_key: String,
    /// The raw 32-byte private key, hex encoded.
    pub private_key: String,
    /// The private key in Wallet Import Format.
    pub wif: String,
}

/// One derivation template's worth of addresses in the `all_address_types` output.
#[derive(Debug, Clone, Serialize)]
pub struct TemplateGroup {
    /// The template name exactly as configured, for example `m-44h-0h-0h-0-i`.
    pub template: String,
    /// The address scheme implied by the template's purpose.
    pub address_type: String,
    /// The template rendered as a human readable derivation path.
    pub pattern: String,
    /// The addresses produced for this template.
    pub addresses: Vec<TemplateAddress>,
}

/// The document produced by a brain wallet tool.
#[derive(Debug, Clone, Serialize)]
pub struct BrainWalletReport {
    /// Warning shown with every brain wallet.
    pub description: &'static str,
    /// The uncompressed-key P2PKH address.
    pub address: String,
    /// The 65-byte uncompressed public key, hex encoded.
    pub public_key: String,
    /// The SHA-256 digest of the passphrase, hex encoded.
    pub private_key: String,
    /// The uncompressed private key in Wallet Import Format.
    pub wif: String,
}

/// The document produced by the mnemonic generation tool.
#[derive(Debug, Clone, Serialize)]
pub struct GeneratedMnemonicReport {
    /// The generated words, space separated.
    pub bip39_mnemonic: String,
    /// The derived seed, hex encoded.
    pub bip39_seed: String,
    /// The BIP32 master key of the seed, serialised as `xprv`.
    pub bip32_root_key: String,
}
