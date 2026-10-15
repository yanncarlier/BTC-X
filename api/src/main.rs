use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use bitcoin::consensus::encode::serialize;
use bitcoin::util::amount::Amount;
use bitcoin::{Address, Network, OutPoint, PackedLockTime, Sequence, Transaction, TxIn, TxOut, Txid};
use hex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Mutex;

// Struct to represent an input in the transaction request
#[derive(Deserialize)]
struct TxInputRequest {
    txid: String,  // Transaction ID as a string
    vout: u32,     // Output index
}

// Struct to represent the complete transaction request
#[derive(Deserialize)]
struct CreateTxRequest {
    inputs: Vec<TxInputRequest>,    // List of inputs
    outputs: Vec<TxOutputRequest>,  // List of outputs
}

// Struct to represent an output in the transaction request
#[derive(Deserialize)]
struct TxOutputRequest {
    address: String,  // Bitcoin address
    amount: u64,      // Amount in satoshis
}

// Struct to represent the transaction response
#[derive(Serialize)]
struct TxResponse {
    tx_hex: String,   // Hex-encoded transaction
}

// Application state to hold the Bitcoin network type
struct AppState {
    network: Network,
}

// Handler for the /create_tx endpoint
async fn create_tx(data: web::Data<Mutex<AppState>>, req: web::Json<CreateTxRequest>) -> impl Responder {
    let network = data.lock().unwrap().network;

    // Process transaction inputs
    let mut inputs = Vec::new();
    for input_req in &req.inputs {
        let txid = match Txid::from_str(&input_req.txid) {
            Ok(txid) => txid,
            Err(_) => return HttpResponse::BadRequest().body("Invalid txid"),
        };
        let vout = input_req.vout;
        let input = TxIn {
            previous_output: OutPoint { txid, vout },
            script_sig: bitcoin::Script::new(),  // Empty script for an unsigned transaction
            sequence: Sequence(0xffffffff),      // Default sequence number
            witness: bitcoin::Witness::new(),    // Empty witness for non-segwit
        };
        inputs.push(input);
    }

    // Process transaction outputs
    let mut outputs = Vec::new();
    for output_req in &req.outputs {
        let address = match Address::from_str(&output_req.address) {
            Ok(addr) => addr,
            Err(_) => return HttpResponse::BadRequest().body("Invalid address"),
        };
        // Check if the address network matches the app's network
        if address.network != network {
            return HttpResponse::BadRequest().body("Address network mismatch");
        }
        let amount = Amount::from_sat(output_req.amount);
        let script_pubkey = address.script_pubkey();
        let output = TxOut {
            value: amount.to_sat(),  // Convert amount to satoshis
            script_pubkey,           // Script public key derived from the address
        };
        outputs.push(output);
    }

    // Build the transaction
    let tx = Transaction {
        version: 1,          // Transaction version
        lock_time: PackedLockTime(0),  // No lock time
        input: inputs,       // List of inputs
        output: outputs,     // List of outputs
    };

    // Serialize the transaction to bytes and encode to hex
    let tx_bytes = serialize(&tx);
    let tx_hex = hex::encode(tx_bytes);

    // Return the response as JSON
    HttpResponse::Ok().json(TxResponse { tx_hex })
}

// Main function to set up and run the server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize application state with Bitcoin mainnet
    let app_state = web::Data::new(Mutex::new(AppState {
        network: Network::Bitcoin,
    }));

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())  // Share the app state across requests
            .route("/create_tx", web::post().to(create_tx))  // Define the endpoint
    })
    // .bind("127.0.0.1:8081")?  // Bind to localhost:8081
    .bind("0.0.0.0:8081")? // This ensures the server is accessible from outside the container.
    .run()
    .await  // Run the server
}