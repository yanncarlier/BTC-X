# Bitcoin Transaction API

A simple HTTP API server for creating unsigned Bitcoin transactions.

## Features

- Create unsigned Bitcoin transactions via REST API
- HTTP server using Actix-web
- Endpoint: POST `/create_tx` to create unsigned Bitcoin transactions
- Accepts inputs (txid, vout) and outputs (address, amount)
- Returns hex-encoded transaction
- Binds to `0.0.0.0:8081` by default
- Supports Bitcoin mainnet

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.88 or higher
- Cargo (comes with Rust)

### Build and Run

```bash
# Navigate to the api directory
cd api

# Build and run the server
cargo run --release
```

Or build the binary and run it separately:

```bash
# Build in release mode
cargo build --release

# Run the binary
./target/release/btcx_api
```

The server will start and listen on `http://localhost:8081` (or `0.0.0.0:8081` for external access).

## Usage

### API Endpoint

**POST** `/create_tx`

### Request Body (JSON)

```json
{
  "inputs": [
    {
      "txid": "transaction-id-hex-string",
      "vout": 0
    }
  ],
  "outputs": [
    {
      "address": "bitcoin-address",
      "amount": 100000
    }
  ]
}
```

### Request Parameters

- `inputs` (array): List of transaction inputs
  - `txid` (string): The transaction ID in hex format (64 characters)
  - `vout` (number): The output index to spend
- `outputs` (array): List of transaction outputs
  - `address` (string): Bitcoin address (must match the server's network - mainnet)
  - `amount` (number): Amount in satoshis (1 BTC = 100,000,000 satoshis)

### Response

```json
{
  "tx_hex": "hex-encoded-raw-transaction"
}
```

### Example Request

```
curl -X POST https://btcx-api.fly.dev/create_tx \
  -H "Content-Type: application/json" \
  -d '{
    "inputs": [
      {
        "txid": "5df6e0e2761359d30a8275058e2678ab78211f49fdf87c8ac664586000000000",
        "vout": 0
      }
    ],
    "outputs": [
      {
        "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
        "amount": 100000
      }
    ]
  }'
```



```bash
curl -X POST http://localhost:8081/create_tx \
  -H "Content-Type: application/json" \
  -d '{
    "inputs": [
      {
        "txid": "5df6e0e2761359d30a8275058e2678ab78211f49fdf87c8ac664586000000000",
        "vout": 0
      }
    ],
    "outputs": [
      {
        "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
        "amount": 100000
      }
    ]
  }'
```

### Example Response

```json
{
  "tx_hex": "010000000100000000605864c68a7cf8fd491f2178ab78268e0575820ad3591376e2e0f65d0000000000ffffffff01a0860100000000001976a91462e907b15cbf27d5425399ebf6f0fb50ebb88f1888ac00000000"
}
```

### Test with Multiple Inputs and Outputs

```bash
curl -X POST http://localhost:8081/create_tx \
  -H "Content-Type: application/json" \
  -d '{
    "inputs": [
      {"txid": "5df6e0e2761359d30a8275058e2678ab78211f49fdf87c8ac664586000000000", "vout": 0},
      {"txid": "5df6e0e2761359d30a8275058e2678ab78211f49fdf87c8ac664586000000001", "vout": 1}
    ],
    "outputs": [
      {"address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", "amount": 50000},
      {"address": "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2", "amount": 50000}
    ]
  }'
```

## Docker

### Build the Docker Image

```bash
docker build -t btcx_api -f api/Dockerfile api/
```

### Run the Container

```bash
docker run -p 8081:8081 btcx_api
```

## Fly.io Deployment

### Install Fly.io CLI

```bash
curl -L https://fly.io/install.sh | sh
flyctl auth login
```

### Deploy to Fly.io

Deploy using the provided `fly.toml` configuration:

```bash
flyctl deploy
```

## Project Structure

```
btcx_api/
├── api/
│   ├── Cargo.toml          # Rust dependencies
│   ├── Dockerfile          # Docker configuration
│   ├── fly.toml            # Fly.io deployment configuration
│   └── src/
│       └── main.rs         # API server implementation
└── README.md
```

## Development

- **Dependency Management:** Automated via Dependabot for Cargo packages
- **Dependencies:** All Rust crates defined in respective `Cargo.toml` files
- **Language:** Rust (systems programming language, memory safe)

## Notes

- This API creates **unsigned** transactions only. Signing is not implemented.
- The server validates transaction IDs and Bitcoin addresses.
- All addresses must match the server's configured network (Bitcoin mainnet by default).
- Amounts are in satoshis (1 BTC = 100,000,000 satoshis).

## License

[Add your license information here]

## Contributing

[Add contribution guidelines here]