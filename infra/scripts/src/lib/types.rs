//! Common types for the Bitcoin Tools library

use std::fmt;
use std::str::FromStr;

use bitcoin::{
    psbt, secp256k1, Address, Amount, Network, OutPoint, PrivateKey, PublicKey, ScriptBuf,
    Transaction, Txid,
};
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::Result;

/// Network type for Bitcoin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BtcNetwork {
    /// Bitcoin mainnet
    Bitcoin,
    /// Bitcoin testnet3
    Testnet,
    /// Bitcoin signet
    Signet,
    /// Bitcoin regtest
    Regtest,
}

impl Default for BtcNetwork {
    fn default() -> Self {
        BtcNetwork::Bitcoin
    }
}

impl From<BtcNetwork> for Network {
    fn from(network: BtcNetwork) -> Self {
        match network {
            BtcNetwork::Bitcoin => Network::Bitcoin,
            BtcNetwork::Testnet => Network::Testnet,
            BtcNetwork::Signet => Network::Signet,
            BtcNetwork::Regtest => Network::Regtest,
        }
    }
}

impl From<Network> for BtcNetwork {
    fn from(network: Network) -> Self {
        match network {
            Network::Bitcoin => BtcNetwork::Bitcoin,
            Network::Testnet => BtcNetwork::Testnet,
            Network::Signet => BtcNetwork::Signet,
            Network::Regtest => BtcNetwork::Regtest,
            _ => BtcNetwork::Bitcoin,
        }
    }
}

impl FromStr for BtcNetwork {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "mainnet" | "bitcoin" | "btc" => Ok(BtcNetwork::Bitcoin),
            "testnet" | "testnet3" => Ok(BtcNetwork::Testnet),
            "signet" => Ok(BtcNetwork::Signet),
            "regtest" => Ok(BtcNetwork::Regtest),
            _ => Err(Error::InvalidNetwork(s.to_string())),
        }
    }
}

impl fmt::Display for BtcNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BtcNetwork::Bitcoin => write!(f, "bitcoin"),
            BtcNetwork::Testnet => write!(f, "testnet"),
            BtcNetwork::Signet => write!(f, "signet"),
            BtcNetwork::Regtest => write!(f, "regtest"),
        }
    }
}

/// UTXO (Unspent Transaction Output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Utxo {
    /// The transaction ID of the output
    pub txid: Txid,
    /// The index of the output in the transaction
    pub vout: u32,
    /// The amount in satoshis
    #[serde(with = "bitcoin::amount::serde::as_sat")]
    pub amount: Amount,
    /// The script that locks the output
    pub script_pubkey: ScriptBuf,
    /// The address that receives the output (if known) as a string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// The number of confirmations (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmations: Option<u32>,
    /// The block height when this UTXO was created (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_height: Option<u32>,
    /// Whether the UTXO is spendable
    #[serde(default = "default_spendable")]
    pub spendable: bool,
}

fn default_spendable() -> bool {
    true
}

impl Utxo {
    /// Create a new UTXO
    pub fn new(
        txid: Txid,
        vout: u32,
        amount: Amount,
        script_pubkey: ScriptBuf,
        address: Option<String>,
    ) -> Self {
        Utxo {
            txid,
            vout,
            amount,
            script_pubkey,
            address,
            confirmations: None,
            block_height: None,
            spendable: true,
        }
    }

    /// Get the outpoint (txid + vout)
    pub fn outpoint(&self) -> OutPoint {
        OutPoint {
            txid: self.txid,
            vout: self.vout,
        }
    }
}

/// Transaction input for signing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningInput {
    /// The transaction ID of the previous output
    pub txid: Txid,
    /// The index of the previous output
    pub vout: u32,
    /// The amount of the previous output
    #[serde(with = "bitcoin::amount::serde::as_sat")]
    pub amount: Amount,
    /// The script that locks the previous output
    pub script_pubkey: ScriptBuf,
    /// The redeem script (for P2SH)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redeem_script: Option<ScriptBuf>,
    /// The witness script (for P2WSH)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness_script: Option<ScriptBuf>,
    /// The sequence number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u32>,
}

impl SigningInput {
    /// Create a new signing input
    pub fn new(txid: Txid, vout: u32, amount: Amount, script_pubkey: ScriptBuf) -> Self {
        SigningInput {
            txid,
            vout,
            amount,
            script_pubkey,
            redeem_script: None,
            witness_script: None,
            sequence: None,
        }
    }

    /// Set the redeem script
    pub fn with_redeem_script(mut self, redeem_script: ScriptBuf) -> Self {
        self.redeem_script = Some(redeem_script);
        self
    }

    /// Set the witness script
    pub fn with_witness_script(mut self, witness_script: ScriptBuf) -> Self {
        self.witness_script = Some(witness_script);
        self
    }

    /// Set the sequence number
    pub fn with_sequence(mut self, sequence: u32) -> Self {
        self.sequence = Some(sequence);
        self
    }
}

/// Coin selection strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoinSelectionStrategy {
    /// Select the smallest UTXOs first (maximizes privacy)
    SmallestFirst,
    /// Select the largest UTXOs first (minimizes fees)
    LargestFirst,
    /// Select UTXOs randomly (good for privacy)
    Random,
    /// Use branch and bound algorithm for exact matches
    BranchAndBound,
}

impl Default for CoinSelectionStrategy {
    fn default() -> Self {
        CoinSelectionStrategy::BranchAndBound
    }
}

impl FromStr for CoinSelectionStrategy {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "smallest_first" => Ok(CoinSelectionStrategy::SmallestFirst),
            "largest_first" => Ok(CoinSelectionStrategy::LargestFirst),
            "random" => Ok(CoinSelectionStrategy::Random),
            "branch_and_bound" | "bnb" => Ok(CoinSelectionStrategy::BranchAndBound),
            _ => Err(Error::Custom(format!(
                "Unknown coin selection strategy: {}",
                s
            ))),
        }
    }
}

impl fmt::Display for CoinSelectionStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoinSelectionStrategy::SmallestFirst => write!(f, "smallest_first"),
            CoinSelectionStrategy::LargestFirst => write!(f, "largest_first"),
            CoinSelectionStrategy::Random => write!(f, "random"),
            CoinSelectionStrategy::BranchAndBound => write!(f, "branch_and_bound"),
        }
    }
}

/// Fee estimation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FeeEstimate {
    /// Fee rate in satoshis per virtual byte
    pub sat_per_vbyte: f32,
    /// Target number of blocks for confirmation
    pub blocks: u32,
}

impl Default for FeeEstimate {
    fn default() -> Self {
        FeeEstimate {
            sat_per_vbyte: 1.0,
            blocks: 6,
        }
    }
}

/// Transaction output target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputTarget {
    /// The destination address as a string
    pub address: String,
    /// The amount to send
    #[serde(with = "bitcoin::amount::serde::as_sat")]
    pub amount: Amount,
    /// Whether this is a change output
    #[serde(default)]
    pub is_change: bool,
}

impl OutputTarget {
    /// Create a new output target
    pub fn new(address: String, amount: Amount) -> Self {
        OutputTarget {
            address,
            amount,
            is_change: false,
        }
    }

    /// Create a new change output target
    pub fn new_change(address: String, amount: Amount) -> Self {
        OutputTarget {
            address,
            amount,
            is_change: true,
        }
    }
}

/// Transaction builder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxBuilderConfig {
    /// The network to use
    pub network: BtcNetwork,
    /// The fee rate in satoshis per virtual byte
    pub fee_rate: f32,
    /// The dust threshold in satoshis
    pub dust_limit: u64,
    /// Whether to use RBF (Replace-By-Fee)
    pub rbf: bool,
    /// The sequence number to use for RBF
    pub rbf_sequence: u32,
    /// The minimum change amount to keep as change (otherwise add to fee)
    #[serde(with = "bitcoin::amount::serde::as_sat")]
    pub min_change: Amount,
    /// The coin selection strategy to use
    pub coin_selection: CoinSelectionStrategy,
}

impl Default for TxBuilderConfig {
    fn default() -> Self {
        TxBuilderConfig {
            network: BtcNetwork::Bitcoin,
            fee_rate: 1.0,
            dust_limit: 546, // Standard dust limit
            rbf: false,
            rbf_sequence: 0xFFFFFFFD,            // Enable RBF with nSequence
            min_change: Amount::from_sat(1_000), // 0.00001 BTC
            coin_selection: CoinSelectionStrategy::default(),
        }
    }
}

/// Transaction signing options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningOptions {
    /// Whether to sign with SIGHASH_ALL
    pub sighash_all: bool,
    /// Whether to sign with SIGHASH_NONE
    pub sighash_none: bool,
    /// Whether to sign with SIGHASH_SINGLE
    pub sighash_single: bool,
    /// Whether to use SIGHASH_ANYONECANPAY
    pub sighash_anyone_can_pay: bool,
    /// Whether to sign with SIGHASH_DEFAULT (for Taproot)
    pub sighash_default: bool,
}

impl Default for SigningOptions {
    fn default() -> Self {
        SigningOptions {
            sighash_all: true,
            sighash_none: false,
            sighash_single: false,
            sighash_anyone_can_pay: false,
            sighash_default: false,
        }
    }
}

/// A signed transaction with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    /// The raw transaction
    pub tx: Transaction,
    /// The transaction ID
    pub txid: Txid,
    /// The transaction size in bytes
    pub size: usize,
    /// The transaction virtual size (vsize)
    pub vsize: usize,
    /// The transaction weight
    pub weight: usize,
    /// The transaction fee in satoshis
    #[serde(with = "bitcoin::amount::serde::as_sat")]
    pub fee: Amount,
    /// The fee rate in satoshis per virtual byte
    pub fee_rate: f32,
    /// Whether the transaction is fully signed
    pub is_complete: bool,
}

impl SignedTransaction {
    /// Create a new signed transaction
    pub fn new(tx: Transaction, fee: Amount, is_complete: bool) -> Result<Self> {
        let txid = tx.txid();
        let weight = tx.weight().to_wu() as usize;
        let vsize = (weight + 3) / 4; // Convert weight to vsize (rounded up)
        let size = tx.size();
        let fee_rate = fee.to_btc() / (vsize as f64 / 100_000_000.0);

        Ok(SignedTransaction {
            tx,
            txid,
            size,
            vsize,
            weight,
            fee,
            fee_rate: fee_rate as f32,
            is_complete,
        })
    }

    /// Get the transaction as hex
    pub fn to_hex(&self) -> String {
        bitcoin::consensus::encode::serialize_hex(&self.tx)
    }
}

/// A partially signed transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartiallySignedTransaction {
    /// The PSBT (Partially Signed Bitcoin Transaction)
    pub psbt: psbt::PartiallySignedTransaction,
    /// Whether the transaction is fully signed
    pub is_complete: bool,
}

impl PartiallySignedTransaction {
    /// Create a new PSBT
    pub fn new(psbt: psbt::PartiallySignedTransaction) -> Self {
        // TODO: implement proper check for finalized PSBT
        let is_complete = false;
        PartiallySignedTransaction { psbt, is_complete }
    }

    /// Get the PSBT as hex
    pub fn to_hex(&self) -> Result<String> {
        let bytes = self.psbt.serialize();
        Ok(hex::encode(&bytes))
    }

    /// Create a PSBT from hex
    pub fn from_hex(s: &str) -> Result<Self> {
        let bytes = hex::decode(s).map_err(|e| Error::Custom(format!("Invalid hex: {}", e)))?;
        let psbt = psbt::PartiallySignedTransaction::deserialize(&bytes)
            .map_err(|e| Error::Custom(format!("Invalid PSBT: {}", e)))?;
        Ok(PartiallySignedTransaction::new(psbt))
    }
}

/// A key pair (private key and public key)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    /// The private key
    pub private_key: PrivateKey,
    /// The public key
    pub public_key: PublicKey,
    /// The network
    pub network: BtcNetwork,
}

impl KeyPair {
    /// Create a new key pair from a private key
    pub fn from_private_key(private_key: PrivateKey, network: BtcNetwork) -> Self {
        let public_key = private_key.public_key(&secp256k1::Secp256k1::new());
        KeyPair {
            private_key,
            public_key,
            network,
        }
    }

    /// Create a new random key pair
    pub fn new(network: BtcNetwork) -> Self {
        let secp = secp256k1::Secp256k1::new();
        let (secret_key, _) = secp.generate_keypair(&mut secp256k1::rand::thread_rng());
        let private_key = PrivateKey::new(secret_key, network.into());
        Self::from_private_key(private_key, network)
    }

    /// Get the address for this key pair
    pub fn address(&self, address_type: &AddressType) -> Result<Address> {
        match address_type {
            AddressType::P2pkh => Ok(Address::p2pkh(&self.public_key, self.network.into())),
            AddressType::P2shP2wpkh => {
                // Use the public key directly, let the function compute the hash
                Ok(Address::p2shwpkh(&self.public_key, self.network.into())?)
            }
            AddressType::P2wpkh => Ok(Address::p2wpkh(&self.public_key, self.network.into())?),
            AddressType::P2tr => {
                // For Taproot, we need an internal key and no script tree for now
                let internal_key = self.public_key;
                // Convert PublicKey to XOnlyPublicKey
                let (x_only, _) = internal_key.inner.x_only_public_key();
                Ok(Address::p2tr(
                    &secp256k1::Secp256k1::new(),
                    x_only,
                    None,
                    self.network.into(),
                ))
            }
        }
    }
}

/// Address type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddressType {
    /// Pay-to-Public-Key-Hash (P2PKH)
    P2pkh,
    /// Nested Pay-to-Witness-Public-Key-Hash (P2SH-P2WPKH)
    P2shP2wpkh,
    /// Native SegWit Pay-to-Witness-Public-Key-Hash (P2WPKH)
    P2wpkh,
    /// Taproot (P2TR)
    P2tr,
}

impl Default for AddressType {
    fn default() -> Self {
        AddressType::P2wpkh // Default to native SegWit
    }
}

impl FromStr for AddressType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "p2pkh" => Ok(AddressType::P2pkh),
            "p2sh-p2wpkh" | "p2sh" => Ok(AddressType::P2shP2wpkh),
            "p2wpkh" | "wpkh" => Ok(AddressType::P2wpkh),
            "p2tr" | "tr" => Ok(AddressType::P2tr),
            _ => Err(Error::Custom(format!("Unknown address type: {}", s))),
        }
    }
}

impl fmt::Display for AddressType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressType::P2pkh => write!(f, "p2pkh"),
            AddressType::P2shP2wpkh => write!(f, "p2sh-p2wpkh"),
            AddressType::P2wpkh => write!(f, "p2wpkh"),
            AddressType::P2tr => write!(f, "p2tr"),
        }
    }
}
