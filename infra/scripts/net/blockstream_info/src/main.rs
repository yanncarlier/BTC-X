use reqwest::blocking::get;
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Deserialize)]
struct AddressInfo {
    chain_stats: ChainStats,
}

#[derive(Deserialize)]
struct ChainStats {
    funded_txo_sum: u64,
    spent_txo_sum: u64,
}

fn main() -> io::Result<()> {
    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();

    // Handle help flag
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        println!("Usage: {} [file_name]", args[0]);
        println!();
        println!("Reads Bitcoin addresses from file_name (one per line) and checks their balance.");
        println!("If no file_name is provided or file_name is '-', reads from stdin.");
        println!("Exits when a non-zero balance is found.");
        return Ok(());
    }

    // Determine input source: if no argument or argument is "-", read from stdin
    let input_source: Box<dyn BufRead> = if args.len() < 2 || args[1] == "-" {
        // Read from stdin
        Box::new(BufReader::new(io::stdin()))
    } else {
        // Open the file specified by the argument
        let file = File::open(&args[1])?;
        Box::new(BufReader::new(file))
    };

    // Loop over each line in the input source
    for line in input_source.lines() {
        let address = line?;
        let address = address.trim(); // Remove whitespace

        // Skip empty lines
        if address.is_empty() {
            continue;
        }

        // Construct the API URL
        let url = format!("https://blockstream.info/api/address/{}", address);

        // Make the API request and handle errors
        match get(&url) {
            Ok(response) => {
                match response.json::<AddressInfo>() {
                    Ok(address_info) => {
                        let balance = address_info.chain_stats.funded_txo_sum
                            - address_info.chain_stats.spent_txo_sum;
                        println!("Address: {}, Balance: {} satoshis", address, balance);
                        if balance > 0 {
                            break; // Stop processing further addresses
                        }
                    }
                    Err(e) => eprintln!("Error parsing JSON for address {}: {}", address, e),
                }
            }
            Err(e) => eprintln!("Error fetching data for address {}: {}", address, e),
        }
    }

    Ok(())
}
