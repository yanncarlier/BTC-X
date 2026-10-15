use bitcoin::consensus::encode::{deserialize, serialize};
use bitcoin::util::key::PrivateKey;
use bitcoin::{Address, Network, Script, Transaction};
use bitcoin_hashes::Hash;
use hex;
use secp256k1::{Message, Secp256k1};
use serde::Deserialize;
use std::io::{self, Read};
use std::str::FromStr;

#[derive(Deserialize)]
struct SignInput {
    private_key_wif: String,
    address: String, // Address corresponding to the input (used to derive scriptPubKey)
}

#[derive(Deserialize)]
struct SignTxRequest {
    unsigned_tx_hex: String,
    inputs: Vec<SignInput>, // One entry per input in the transaction
}

// Helper function to push data onto a script (manual script building)
fn push_data(bytes: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let len = bytes.len();
    if len < 76 {
        result.push(len as u8);
    } else if len < 256 {
        result.push(76u8); // OP_PUSHDATA1
        result.push(len as u8);
    } else {
        result.push(77u8); // OP_PUSHDATA2
        result.push((len & 0xff) as u8);
        result.push((len >> 8) as u8);
    }
    result.extend_from_slice(bytes);
    result
}

fn sign_transaction(request: SignTxRequest, network: Network) -> Result<String, String> {
    // Deserialize the unsigned transaction
    let tx_bytes =
        hex::decode(&request.unsigned_tx_hex).map_err(|e| format!("Invalid hex: {}", e))?;
    let mut tx: Transaction =
        deserialize(&tx_bytes).map_err(|e| format!("Failed to deserialize transaction: {}", e))?;

    // Validate input count matches
    if request.inputs.len() != tx.input.len() {
        return Err(format!(
            "Input count mismatch: transaction has {} inputs, but {} signing inputs provided",
            tx.input.len(),
            request.inputs.len()
        ));
    }

    let secp = Secp256k1::new();

    // Sign each input
    for (i, sign_input) in request.inputs.iter().enumerate() {
        // Parse the private key from WIF
        let privkey = PrivateKey::from_wif(&sign_input.private_key_wif)
            .map_err(|e| format!("Invalid WIF for input {}: {}", i, e))?;

        // Verify network matches
        if privkey.network != network {
            return Err(format!(
                "Private key network mismatch for input {}: {:?} vs {:?}",
                i, privkey.network, network
            ));
        }

        // Parse the address to get scriptPubKey
        let address = Address::from_str(&sign_input.address)
            .map_err(|e| format!("Invalid address for input {}: {}", i, e))?;

        if address.network != network {
            return Err(format!(
                "Address network mismatch for input {}: {:?} vs {:?}",
                i, address.network, network
            ));
        }

        let script_pubkey = address.script_pubkey();

        // Get the secret key
        let secret_key = privkey.inner;

        // Compute the signature hash (SIGHASH_ALL = 0x01)
        let sighash = tx.signature_hash(i, &script_pubkey, 0x01);

        // Create a message from the sighash
        let msg = Message::from_slice(&sighash.as_hash().into_inner())
            .map_err(|e| format!("Failed to create message for input {}: {}", i, e))?;

        // Sign the message
        let sig = secp.sign_ecdsa(&msg, &secret_key);

        // Get the public key
        let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
        let public_key_bytes = public_key.serialize();

        // Create scriptSig: <signature> <public_key>
        // Signature needs SIGHASH_ALL byte appended (0x01)
        let mut sig_bytes = sig.serialize_der().to_vec();
        sig_bytes.push(0x01); // Append SIGHASH_ALL

        // Build scriptSig bytes manually
        let mut script_sig_bytes = Vec::new();
        script_sig_bytes.extend_from_slice(&push_data(&sig_bytes));
        script_sig_bytes.extend_from_slice(&push_data(&public_key_bytes));

        // Create Script from bytes
        let script_sig = Script::from(script_sig_bytes);

        // Update the transaction input with the scriptSig
        tx.input[i].script_sig = script_sig;
    }

    // Serialize the signed transaction
    let signed_tx_bytes = serialize(&tx);
    let signed_tx_hex = hex::encode(signed_tx_bytes);

    Ok(signed_tx_hex)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Determine if we should read from stdin or use command-line argument
    let json_input = if args.len() > 1 {
        args[1].clone()
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Failed to read from stdin");
        buffer
    };

    // Parse JSON input
    let request: SignTxRequest = match serde_json::from_str(&json_input) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
            eprintln!("Usage: {} [json_input]", args[0]);
            eprintln!("Example JSON:");
            eprintln!(
                r#"{{"unsigned_tx_hex": "...", "inputs": [{{"private_key_wif": "5K...", "address": "1A1z..."}}]}}"#
            );
            std::process::exit(1);
        }
    };

    // Use Bitcoin mainnet
    let network = Network::Bitcoin;

    // Sign the transaction
    match sign_transaction(request, network) {
        Ok(signed_tx_hex) => {
            println!("{}", signed_tx_hex);
        }
        Err(e) => {
            eprintln!("Error signing transaction: {}", e);
            std::process::exit(1);
        }
    }
}
