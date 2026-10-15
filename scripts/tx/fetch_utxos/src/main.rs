use reqwest::blocking::get;
use serde::{Deserialize, Serialize};
use std::env;
use std::io;
use std::process;

#[derive(Deserialize, Serialize, Debug)]
struct Utxo {
    txid: String,
    vout: u32,
    status: UtxoStatus,
    value: u64,
    #[serde(default)]
    address: Option<String>,
    #[serde(default)]
    script_pubkey: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct UtxoStatus {
    confirmed: bool,
    #[serde(default)]
    block_height: Option<u64>,
    #[serde(default)]
    block_hash: Option<String>,
    #[serde(default)]
    block_time: Option<u64>,
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <address>", args[0]);
        process::exit(1);
    }

    let address = args[1].trim();
    let url = format!("https://blockstream.info/api/address/{}/utxo", address);

    match get(&url) {
        Ok(response) => {
            if !response.status().is_success() {
                eprintln!(
                    "Error: HTTP {} - Failed to fetch UTXOs for address {}",
                    response.status(),
                    address
                );
                process::exit(1);
            }

            match response.json::<Vec<Utxo>>() {
                Ok(utxos) => {
                    let json = serde_json::to_string_pretty(&utxos).unwrap();
                    println!("{}", json);
                }
                Err(e) => {
                    eprintln!("Error parsing JSON for address {}: {}", address, e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching data for address {}: {}", address, e);
            process::exit(1);
        }
    }

    Ok(())
}
