use reqwest::blocking::get;
use serde::Deserialize;
use std::env;
use std::process;

#[derive(Deserialize, serde::Serialize)]
struct TxInput {
    txid: String,
    vout: u32,
    prevout: Option<TxOutput>,
    scriptsig: String,
    scriptsig_asm: String,
    witness: Option<Vec<String>>,
    sequence: u64,
}

#[derive(Deserialize, serde::Serialize)]
struct TxOutput {
    scriptpubkey: String,
    scriptpubkey_asm: String,
    scriptpubkey_type: String,
    scriptpubkey_address: Option<String>,
    value: u64,
}

#[derive(Deserialize, serde::Serialize)]
struct TransactionInfo {
    txid: String,
    version: u32,
    locktime: u32,
    vin: Vec<TxInput>,
    vout: Vec<TxOutput>,
    size: u32,
    weight: u32,
    fee: u64,
    status: TxStatus,
}

#[derive(Deserialize, serde::Serialize)]
struct TxStatus {
    confirmed: bool,
    block_height: Option<u64>,
    block_hash: Option<String>,
    block_time: Option<u64>,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <txid>", args[0]);
        eprintln!("Example: {} abc123def456...", args[0]);
        process::exit(1);
    }

    let txid = args[1].trim();

    if txid.is_empty() {
        eprintln!("Error: txid cannot be empty");
        process::exit(1);
    }

    // Construct the API URL
    let url = format!("https://blockstream.info/api/tx/{}", txid);

    // Make the API request
    match get(&url) {
        Ok(response) => {
            if !response.status().is_success() {
                eprintln!(
                    "Error: HTTP {} - Transaction not found or API error",
                    response.status()
                );
                process::exit(1);
            }

            match response.json::<TransactionInfo>() {
                Ok(tx_info) => {
                    // Print transaction details in a readable format
                    println!("Transaction ID: {}", tx_info.txid);
                    println!("Version: {}", tx_info.version);
                    println!("Locktime: {}", tx_info.locktime);
                    println!("Size: {} bytes", tx_info.size);
                    println!("Weight: {}", tx_info.weight);
                    println!("Fee: {} satoshis", tx_info.fee);

                    // Status information
                    if tx_info.status.confirmed {
                        println!("Status: Confirmed");
                        if let Some(height) = tx_info.status.block_height {
                            println!("Block Height: {}", height);
                        }
                        if let Some(ref block_hash) = tx_info.status.block_hash {
                            println!("Block Hash: {}", block_hash);
                        }
                        if let Some(block_time) = tx_info.status.block_time {
                            println!("Block Time: {}", block_time);
                        }
                    } else {
                        println!("Status: Unconfirmed");
                    }

                    // Inputs
                    println!("\nInputs ({}):", tx_info.vin.len());
                    for (i, input) in tx_info.vin.iter().enumerate() {
                        println!("  Input {}:", i);
                        println!("    Previous TXID: {}", input.txid);
                        println!("    Vout: {}", input.vout);
                        println!("    Sequence: {}", input.sequence);
                        if let Some(ref prevout) = input.prevout {
                            if let Some(ref addr) = prevout.scriptpubkey_address {
                                println!("    Address: {}", addr);
                            }
                            println!("    Amount: {} satoshis", prevout.value);
                        }
                    }

                    // Outputs
                    println!("\nOutputs ({}):", tx_info.vout.len());
                    for (i, output) in tx_info.vout.iter().enumerate() {
                        println!("  Output {}:", i);
                        if let Some(ref addr) = output.scriptpubkey_address {
                            println!("    Address: {}", addr);
                        }
                        println!("    Amount: {} satoshis", output.value);
                        println!("    Script Type: {}", output.scriptpubkey_type);
                    }

                    // Also output as JSON for programmatic use
                    if let Ok(json) = serde_json::to_string_pretty(&tx_info) {
                        println!("\nJSON Output:");
                        println!("{}", json);
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing JSON response: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching transaction data: {}", e);
            process::exit(1);
        }
    }
}
