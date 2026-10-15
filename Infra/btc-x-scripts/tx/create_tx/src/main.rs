use bitcoin::consensus::encode::serialize;
use bitcoin::util::amount::Amount;
use bitcoin::{
    Address, Network, OutPoint, PackedLockTime, Sequence, Transaction, TxIn, TxOut, Txid,
};
use hex;
use serde::Deserialize;
use std::io::{self, Read};
use std::str::FromStr;

// Struct to represent an input in the transaction request
#[derive(Deserialize)]
struct TxInputRequest {
    txid: String, // Transaction ID as a string
    vout: u32,    // Output index
}

// Struct to represent the complete transaction request
#[derive(Deserialize)]
struct CreateTxRequest {
    inputs: Vec<TxInputRequest>,   // List of inputs
    outputs: Vec<TxOutputRequest>, // List of outputs
}

// Struct to represent an output in the transaction request
#[derive(Deserialize)]
struct TxOutputRequest {
    address: String, // Bitcoin address
    amount: u64,     // Amount in satoshis
}

fn create_transaction(request: CreateTxRequest, network: Network) -> Result<String, String> {
    // Process transaction inputs
    let mut inputs = Vec::new();
    for input_req in &request.inputs {
        let txid = match Txid::from_str(&input_req.txid) {
            Ok(txid) => txid,
            Err(_) => return Err(format!("Invalid txid: {}", input_req.txid)),
        };
        let vout = input_req.vout;
        let input = TxIn {
            previous_output: OutPoint { txid, vout },
            script_sig: bitcoin::Script::new(), // Empty script for an unsigned transaction
            sequence: Sequence(0xffffffff),     // Default sequence number
            witness: bitcoin::Witness::new(),   // Empty witness for non-segwit
        };
        inputs.push(input);
    }

    // Process transaction outputs
    let mut outputs = Vec::new();
    for output_req in &request.outputs {
        let address = match Address::from_str(&output_req.address) {
            Ok(addr) => addr,
            Err(e) => return Err(format!("Invalid address {}: {}", output_req.address, e)),
        };
        // Check if the address network matches the requested network
        if address.network != network {
            return Err(format!(
                "Address network mismatch: address is for {:?}, but requested {:?}",
                address.network, network
            ));
        }
        let amount = Amount::from_sat(output_req.amount);
        let script_pubkey = address.script_pubkey();
        let output = TxOut {
            value: amount.to_sat(), // Convert amount to satoshis
            script_pubkey,          // Script public key derived from the address
        };
        outputs.push(output);
    }

    // Build the transaction
    let tx = Transaction {
        version: 1,                   // Transaction version
        lock_time: PackedLockTime(0), // No lock time
        input: inputs,                // List of inputs
        output: outputs,              // List of outputs
    };

    // Serialize the transaction to bytes and encode to hex
    let tx_bytes = serialize(&tx);
    let tx_hex = hex::encode(tx_bytes);

    Ok(tx_hex)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Determine if we should read from stdin or use command-line argument
    let json_input = if args.len() > 1 {
        // Use command-line argument
        args[1].clone()
    } else {
        // Read from stdin
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Failed to read from stdin");
        buffer
    };

    // Parse JSON input
    let request: CreateTxRequest = match serde_json::from_str(&json_input) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
            eprintln!("Usage: {} [json_input]", args[0]);
            eprintln!("Example JSON:");
            eprintln!(
                r#"{{"inputs": [{{"txid": "abc123...", "vout": 0}}], "outputs": [{{"address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", "amount": 1000}}]}}"#
            );
            std::process::exit(1);
        }
    };

    // Validate that we have at least one input and one output
    if request.inputs.is_empty() {
        eprintln!("Error: At least one input is required");
        std::process::exit(1);
    }

    if request.outputs.is_empty() {
        eprintln!("Error: At least one output is required");
        std::process::exit(1);
    }

    // Use Bitcoin mainnet (can be extended to support testnet/regtest if needed)
    let network = Network::Bitcoin;

    // Create the transaction
    match create_transaction(request, network) {
        Ok(tx_hex) => {
            println!("{}", tx_hex);
        }
        Err(e) => {
            eprintln!("Error creating transaction: {}", e);
            std::process::exit(1);
        }
    }
}
