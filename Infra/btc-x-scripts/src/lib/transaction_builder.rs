//! Transaction builder for creating and signing Bitcoin transactions

use std::str::FromStr;

use bitcoin::{
    absolute, secp256k1, Address, Amount, EcdsaSighashType, OutPoint, Script,
    Sequence, Transaction, TxIn, TxOut, Txid, Witness,
};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::types::{
    BtcNetwork, CoinSelectionStrategy, OutputTarget, SigningInput,
    SignedTransaction, Utxo,
};

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
    pub min_change: Amount,
    /// The coin selection strategy to use
    pub coin_selection: CoinSelectionStrategy,
    /// Whether to shuffle inputs for privacy
    pub shuffle_inputs: bool,
    /// Whether to shuffle outputs for privacy
    pub shuffle_outputs: bool,
}

impl Default for TxBuilderConfig {
    fn default() -> Self {
        TxBuilderConfig {
            network: BtcNetwork::Bitcoin,
            fee_rate: 1.0,
            dust_limit: 546, // Standard dust limit
            rbf: false,
            rbf_sequence: 0xFFFFFFFD, // Enable RBF with nSequence
            min_change: Amount::from_sat(1_000), // 0.00001 BTC
            coin_selection: CoinSelectionStrategy::BranchAndBound,
            shuffle_inputs: true,
            shuffle_outputs: true,
        }
    }
}

/// Transaction builder
pub struct TransactionBuilder {
    config: TxBuilderConfig,
    utxos: Vec<Utxo>,
    outputs: Vec<OutputTarget>,
    inputs: Vec<SigningInput>,
    change_address: Option<Address>,
    lock_time: Option<u32>,
    version: i32,
}

impl TransactionBuilder {
    /// Create a new transaction builder with default configuration
    pub fn new(network: BtcNetwork) -> Self {
        TransactionBuilder {
            config: TxBuilderConfig {
                network,
                ..Default::default()
            },
            utxos: Vec::new(),
            outputs: Vec::new(),
            inputs: Vec::new(),
            change_address: None,
            lock_time: None,
            version: 2, // Default to version 2 for BIP68
        }
    }

    /// Set the transaction builder configuration
    pub fn with_config(mut self, config: TxBuilderConfig) -> Self {
        self.config = config;
        self
    }

    /// Add UTXOs to spend
    pub fn with_utxos(mut self, utxos: Vec<Utxo>) -> Self {
        self.utxos = utxos;
        self
    }

    /// Add an output to the transaction
    pub fn add_output(&mut self, address: Address, amount: Amount) -> &mut Self {
        self.outputs.push(OutputTarget::new(address, amount));
        self
    }

    /// Add multiple outputs to the transaction
    pub fn add_outputs(&mut self, outputs: Vec<OutputTarget>) -> &mut Self {
        self.outputs.extend(outputs);
        self
    }

    /// Set the change address
    pub fn set_change_address(&mut self, address: Address) -> &mut Self {
        self.change_address = Some(address);
        self
    }

    /// Set the lock time
    pub fn set_lock_time(&mut self, lock_time: u32) -> &mut Self {
        self.lock_time = Some(lock_time);
        self
    }

    /// Set the transaction version
    pub fn set_version(&mut self, version: i32) -> &mut Self {
        self.version = version;
        self
    }

    /// Build an unsigned transaction
    pub fn build_unsigned(&self) -> Result<Transaction> {
        if self.outputs.is_empty() {
            return Err(Error::Custom("No outputs specified".into()));
        }

        // Select UTXOs
        let selected_utxos = self.select_utxos()?;
        let total_input = selected_utxos.iter().map(|u| u.amount).sum::<Amount>();
        
        // Calculate total output amount
        let total_output = self
            .outputs
            .iter()
            .filter(|o| !o.is_change)
            .map(|o| o.amount)
            .sum::<Amount>();

        // Calculate fee
        let tx = self.create_unsigned_tx(&selected_utxos, None)?;
        let weight = tx.weight().to_wu() as usize;
        let tx_vsize = ((weight + 3) / 4) as u64;
        let fee = (tx_vsize as f32 * self.config.fee_rate).ceil() as u64;
        
        // Check if we have enough funds
        if total_input < total_output + Amount::from_sat(fee) {
            return Err(Error::InsufficientFunds);
        }

        // Calculate change
        let change_amount = total_input - total_output - Amount::from_sat(fee);
        
        // Add change output if needed
        let mut final_tx = if change_amount >= self.config.min_change {
            let change_address = self.change_address.clone().ok_or_else(|| {
                Error::Custom("Change address not specified".into())
            })?;
            
            let mut outputs = self.outputs.clone();
            outputs.push(OutputTarget::new_change(change_address, change_amount));
            
            self.create_unsigned_tx(&selected_utxos, Some(&outputs[..]))?
        } else {
            tx
        };

        // Set lock time if specified
        if let Some(lock_time) = self.lock_time {
            final_tx.lock_time = absolute::LockTime::from(lock_time);
        }

        // Set version
        final_tx.version = self.version;

        Ok(final_tx)
    }

    /// Build and sign a transaction
    pub fn build_signed<F>(&self, signer: F) -> Result<SignedTransaction>
    where
        F: Fn(&Script, u64, &[u8]) -> Result<(Vec<Vec<u8>>, Script)>,
    {
        let unsigned_tx = self.build_unsigned()?;
        let mut signed_tx = unsigned_tx.clone();
        
        // Sign each input
        for (i, input) in signed_tx.input.iter_mut().enumerate() {
            let prevout_script = self.inputs[i].script_pubkey.clone();
            let amount = self.inputs[i].amount;
            
            // Create the signature hash
            let sighash = signed_tx.signature_hash(
                i,
                &prevout_script,
                amount.to_sat(),
                EcdsaSighashType::All,
            )?;
            
            // Get the signatures and witness script
            let (signatures, witness_script) = signer(&prevout_script, amount.to_sat(), &sighash)?;
            
            // Add signatures to the witness
            let mut witness = Witness::new();
            for sig in signatures {
                witness.push(sig);
            }
            witness.push(witness_script.into_bytes());
            input.witness = witness;
        }
        
        // Create signed transaction
        let signed_tx = SignedTransaction::new(signed_tx, self.calculate_fee(&unsigned_tx)?, true)?;
        
        Ok(signed_tx)
    }

    /// Select UTXOs to spend using the configured strategy
    fn select_utxos(&self) -> Result<Vec<Utxo>> {
        match self.config.coin_selection {
            CoinSelectionStrategy::SmallestFirst => self.select_utxos_smallest_first(),
            CoinSelectionStrategy::LargestFirst => self.select_utxos_largest_first(),
            CoinSelectionStrategy::Random => self.select_utxos_random(),
            CoinSelectionStrategy::BranchAndBound => self.select_utxos_branch_and_bound(),
        }
    }

    /// Select UTXOs by smallest first (maximizes privacy)
    fn select_utxos_smallest_first(&self) -> Result<Vec<Utxo>> {
        let mut utxos = self.utxos.clone();
        utxos.sort_by_key(|u| u.amount);
        self.select_utxos_greedy(&utxos)
    }

    /// Select UTXOs by largest first (minimizes fees)
    fn select_utxos_largest_first(&self) -> Result<Vec<Utxo>> {
        let mut utxos = self.utxos.clone();
        utxos.sort_by_key(|u| std::cmp::Reverse(u.amount));
        self.select_utxos_greedy(&utxos)
    }

    /// Select UTXOs randomly (good for privacy)
    fn select_utxos_random(&self) -> Result<Vec<Utxo>> {
        use rand::thread_rng;
        
        let mut utxos = self.utxos.clone();
        let mut rng = thread_rng();
        utxos.shuffle(&mut rng);
        
        self.select_utxos_greedy(&utxos)
    }

    /// Select UTXOs using a greedy algorithm
    fn select_utxos_greedy(&self, sorted_utxos: &[Utxo]) -> Result<Vec<Utxo>> {
        let total_output: Amount = self.outputs.iter().map(|o| o.amount).sum();
        let mut selected = Vec::new();
        let mut total_selected = Amount::from_sat(0);
        
        // Estimate the size of the transaction with a single input and output
        let base_tx_size = 10; // Version + lock_time + input count + output count
        let input_size = 150; // Approximate size of an input
        let output_size = 34; // Approximate size of an output (P2PKH)
        
        // Calculate the minimum amount needed including fees
        let min_amount = total_output + Amount::from_sat(
            (base_tx_size + output_size * self.outputs.len()) as u64 * self.config.fee_rate as u64
        );
        
        for utxo in sorted_utxos {
            if total_selected >= min_amount {
                break;
            }
            
            selected.push(utxo.clone());
            total_selected += utxo.amount;
        }
        
        if total_selected < min_amount {
            return Err(Error::InsufficientFunds);
        }
        
        // Add inputs for signing
        self.inputs = selected
            .iter()
            .map(|utxo| {
                SigningInput::new(
                    utxo.txid,
                    utxo.vout,
                    utxo.amount,
                    utxo.script_pubkey.clone(),
                )
            })
            .collect();
        
        Ok(selected)
    }

    /// Select UTXOs using the branch and bound algorithm (for exact matches)
    fn select_utxos_branch_and_bound(&self) -> Result<Vec<Utxo>> {
        // Implementation of the branch and bound algorithm for coin selection
        // This is a simplified version - a full implementation would be more complex
        
        let target = self
            .outputs
            .iter()
            .filter(|o| !o.is_change)
            .map(|o| o.amount)
            .sum::<Amount>();
        
        // Sort UTXOs by descending amount for better performance
        let mut utxos = self.utxos.clone();
        utxos.sort_by_key(|u| std::cmp::Reverse(u.amount));
        
        let mut best_selection = Vec::new();
        let mut best_amount = Amount::from_sat(0);
        
        // Try to find an exact match
        if let Some(selection) = self.find_exact_match(&utxos, target) {
            return Ok(selection);
        }
        
        // If no exact match, fall back to greedy selection
        self.select_utxos_largest_first()
    }
    
    /// Helper function to find an exact match for the target amount
    fn find_exact_match(&self, utxos: &[Utxo], target: Amount) -> Option<Vec<Utxo>> {
        // This is a simplified version - a full implementation would use dynamic programming
        // or a more sophisticated algorithm for large sets of UTXOs
        
        for i in 0..utxos.len() {
            let mut sum = Amount::from_sat(0);
            let mut selection = Vec::new();
            
            for utxo in &utxos[i..] {
                if sum + utxo.amount <= target {
                    sum += utxo.amount;
                    selection.push(utxo.clone());
                    
                    if sum == target {
                        return Some(selection);
                    }
                }
            }
        }
        
        None
    }

    /// Create an unsigned transaction with the given UTXOs and outputs
    fn create_unsigned_tx(
        &self,
        utxos: &[Utxo],
        outputs: Option<&[OutputTarget]>,
    ) -> Result<Transaction> {
        let outputs = outputs.unwrap_or(&self.outputs);
        
        // Create inputs
        let inputs: Vec<TxIn> = utxos
            .iter()
            .map(|utxo| {
                let sequence = if self.config.rbf {
                    self.config.rbf_sequence
                } else {
                    Sequence::MAX
                };
                
                TxIn {
                    previous_output: OutPoint::new(utxo.txid, utxo.vout),
                    script_sig: Script::new(),
                    sequence: Sequence(sequence),
                    witness: Witness::new(),
                }
            })
            .collect();
        
        // Create outputs
        let outputs: Vec<TxOut> = outputs
            .iter()
            .map(|output| TxOut {
                value: output.amount.to_sat(),
                script_pubkey: output.address.script_pubkey(),
            })
            .collect();
        
        // Create transaction
        let tx = Transaction {
            version: self.version,
            lock_time: absolute::LockTime::ZERO,
            input: inputs,
            output: outputs,
        };
        
        Ok(tx)
    }
    
    /// Calculate the fee for a transaction
    fn calculate_fee(&self, tx: &Transaction) -> Result<Amount> {
        // Calculate the total input amount
        let input_amount: Amount = self.inputs.iter().map(|i| i.amount).sum();
        
        // Calculate the total output amount
        let output_amount: Amount = tx.output.iter().map(|o| Amount::from_sat(o.value)).sum();
        
        // The fee is the difference between inputs and outputs
        Ok(input_amount - output_amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::secp256k1::Secp256k1;
    use bitcoin::PrivateKey;
    
    #[test]
    fn test_transaction_builder() {
        // Create a test network
        let network = BtcNetwork::Regtest;
        
        // Create a test key pair
        let secp = Secp256k1::new();
        let private_key = PrivateKey::new(
            secp256k1::SecretKey::new(&mut rand::thread_rng()),
            network.into(),
        );
        let public_key = private_key.public_key(&secp);
        let address = Address::p2pkh(&public_key, network.into());
        
        // Create a test UTXO
        let utxo = Utxo {
            txid: Txid::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            vout: 0,
            amount: Amount::from_btc(1.0).unwrap(),
            script_pubkey: address.script_pubkey(),
            address: Some(address.clone()),
            confirmations: Some(6),
            block_height: Some(100),
            spendable: true,
        };
        
        // Create a transaction builder
        let mut builder = TransactionBuilder::new(network)
            .with_utxos(vec![utxo])
            .add_output(address.clone(), Amount::from_btc(0.5).unwrap())
            .set_change_address(address);
            
        // Build an unsigned transaction
        let unsigned_tx = builder.build_unsigned();
        assert!(unsigned_tx.is_ok());
        
        // Build and sign the transaction
        let signer = |script: &Script, amount: u64, sighash: &[u8]| {
            // In a real implementation, this would sign the transaction
            Ok((vec![vec![0; 72]], script.clone()))
        };
        
        let signed_tx = builder.build_signed(signer);
        assert!(signed_tx.is_ok());
    }
}
