use reqwest::blocking::get;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::process;

#[derive(Deserialize, Debug)]
struct FeeEstimates {
    #[serde(flatten)]
    estimates: HashMap<String, f64>,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        eprintln!("Usage: {} [blocks]", args[0]);
        eprintln!("  blocks: (optional) target confirmation blocks (e.g., 1, 3, 6, 12, 25)");
        process::exit(1);
    }

    let target_blocks: Option<u32> = args.get(1).and_then(|s| s.parse().ok());

    let url = "https://blockstream.info/api/fee-estimates";
    match get(url) {
        Ok(response) => {
            if !response.status().is_success() {
                eprintln!(
                    "Error: HTTP {} - Failed to fetch fee estimates",
                    response.status()
                );
                process::exit(1);
            }

            match response.json::<FeeEstimates>() {
                Ok(fee_estimates) => {
                    if let Some(target) = target_blocks {
                        // Show fee for a specific target
                        let target_key = target.to_string();
                        if let Some(fee_rate) = fee_estimates.estimates.get(&target_key) {
                            // Convert from sat/vByte (the API returns sat/vByte)
                            println!(
                                "Fee estimate for {} blocks: {:.1} sat/vByte",
                                target, fee_rate
                            );
                        } else {
                            eprintln!("No fee estimate available for {} blocks", target);
                            process::exit(1);
                        }
                    } else {
                        // Show all fee estimates
                        println!("Fee estimates (sat/vByte):");
                        let mut sorted_keys: Vec<&String> =
                            fee_estimates.estimates.keys().collect();
                        sorted_keys.sort_by_key(|k| k.parse::<u32>().unwrap_or(0));
                        for key in sorted_keys {
                            if let Ok(blocks) = key.parse::<u32>() {
                                let fee_rate = fee_estimates.estimates.get(key).unwrap();
                                println!("  {:>3} blocks: {:.1} sat/vByte", blocks, fee_rate);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing fee estimates: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching fee estimates: {}", e);
            process::exit(1);
        }
    }
}
