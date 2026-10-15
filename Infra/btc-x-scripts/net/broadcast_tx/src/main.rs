use reqwest::blocking::Client;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <signed_transaction_hex>", args[0]);
        process::exit(1);
    }

    let tx_hex = args[1].trim();
    if tx_hex.is_empty() {
        eprintln!("Error: transaction hex cannot be empty");
        process::exit(1);
    }

    let client = Client::new();
    let url = "https://blockstream.info/api/tx";

    match client.post(url).body(tx_hex.to_string()).send() {
        Ok(response) => {
            if !response.status().is_success() {
                eprintln!(
                    "Error: HTTP {} - Failed to broadcast transaction: {}",
                    response.status(),
                    response
                        .text()
                        .unwrap_or_else(|_| "unknown error".to_string())
                );
                process::exit(1);
            }

            match response.text() {
                Ok(txid) => {
                    println!("Transaction broadcasted successfully!");
                    println!("Transaction ID (txid): {}", txid);
                }
                Err(e) => {
                    eprintln!("Error reading response: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error broadcasting transaction: {}", e);
            process::exit(1);
        }
    }
}
