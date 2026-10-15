# Bitcoin Tools (btcx_tools)

**Disclaimer: Use at Your Own Risk** This is experimental beta software and may contain bugs or exhibit unexpected behavior. It is provided "as-is" without any warranties. Use of this software is entirely at your own discretion and risk.

A comprehensive collection of Bitcoin utilities written in Rust, featuring command-line tools for Bitcoin address generation, mnemonics, wallet operations, and blockchain queries.

## Prerequisites

- Rust Installation

## Quick Start

### Building & Installation

#### One-command build (recommended)

The project includes a convenient build script that compiles **all** tools in release mode and places the binaries in the `dist/` directory:

```bash
scripts/build/build.sh --release
```

What it does:

- Builds all crates in release mode
- Copies binaries to ./dist/
- Strips debug symbols (if strip is available)
- Applies UPX compression (if upx is installed – optional)

After running the script you will have:

```
dist/
├── all_address_types
├── blockstream_balance_loop
├── blockstream_tx
├── brain_wallet
├── broadcast_tx
├── create_tx
├── estimate_fee
├── fetch_utxos
├── generate_mnemonic
├── m_0_i
├── m_0h_0_0h
├── m_0h_0_i
├── m_0h_0_ih
├── m_0h_0h_0h
├── m_0h_0h_i
├── m_0h_0h_ih
├── m_0h_i
├── m_44h_0h_0h_0
├── m_44h_0h_0h_0_i
├── m_44h_0h_0h_0_ih
├── m_44h_0h_0h_i
├── m_44h_0h_0h_ih
├── m_49h_0h_0h_0_i
├── m_49h_0h_0h_0_ih
├── m_84h_0h_0h_0_i
├── m_84h_0h_0h_0_ih
├── m_86h_0h_0h_i
└── sign_tx
```

#### Alternative: Build manually

Every crate is a workspace member, so a plain workspace build works:

```bash
cargo build --workspace            # debug
cargo build --workspace --release  # release
```

Individual tools can still be built on their own:

```bash
cargo build --release -p generate_mnemonic
cargo build --release -p m_44h_0h_0h_i
cargo build --release -p all_address_types
```

**Recommendation:** Use `scripts/build/build.sh --release` — it's faster and keeps everything organized in one place.

The build script supports several options:

```bash
# Show help and usage information
scripts/build/build.sh --help

# Build debug binaries (default)
scripts/build/build.sh

# Build release binaries (recommended for production)
scripts/build/build.sh --release

# Clean build directories and rebuild
scripts/build/build.sh --clean --release
```

### Basic Usage

```bash
# Brain wallet from passphrase
./dist/brain_wallet "correct horse battery staple"

{
    "description": "Brain wallet (WARNING: Highly insecure, for educational purposes only)",
    "address": "1JwSSubhmg6iPtRjtyqhUYYH7bZg3Lfy1T",
    "public_key": "0478d430274f8c5ec1321338151e9f27f4c676a008bdf8638d07c0b6be9ab35c71a1518063243acd4dfe96b66e3f2ec8013c8e072cd09b3834a19f81f659cc3455",
    "private_key": "c4bbcb1fbec99d65bf59d85c8cb62ee2db963f0fe106f483d9afa73bd4e39a8a",
    "wif": "5KJvsngHeMpm884wtkJNzQGaCErckhHJBGFsvd3UyK5qMZXj3hS"
}
```

```bash
# Generate a 12-word mnemonic, its seed and BIP32 root key
./dist/generate_mnemonic

{
    "bip39_mnemonic": "track welcome right sound ranch lunch bracket match toward surge elder dolphin",
    "bip39_seed": "e43badff...",
    "bip32_root_key": "xprv9s21ZrQH143K3..."
}
```

```bash
# Generate 3 addresses from the default test mnemonic
./dist/m_44h_0h_0h_i -n 3

# ...or supply your own mnemonic
./dist/m_44h_0h_0h_i -n 3 "word1 word2 ... word12"

# ...or use the bare-count / bare-mnemonic forms
./dist/m_44h_0h_0h_i 3
./dist/m_44h_0h_0h_i "word1 word2 ... word12" 3
```

Every `m-...` tool prints a JSON document with the mnemonic, the seed, the
account extended keys and the derived addresses:

```json
{
  "mnemonic_phrase": "abandon abandon ... about",
  "passphrase": "",
  "seed_hex": "5eb00bbd...",
  "address_type": "BIP44 P2PKH (raw m/44'/0'/0'/{i})",
  "account_extended_public_key": "xpub6BosfCnifzxc...",
  "account_extended_private_key": "xprv9xpXFhFpqdQ...",
  "addresses": [
    {
      "index": 0,
      "derivation_path": "m/44'/0'/0'/0",
      "address": "13KE6TffArLh4fVM6uoQzvsYq5vwetJcVM",
      "public_key": "0386b865b52b753d0a84d09bc20063fab5d8453ec33c215d4019a5801c9c6438b9",
      "private_key": "83bda5c7add17ef9bbc1f03391913fe6cc947aa18c4a343607724e815c83eeb7",
      "wif": "L1doE4V5uyQvqMtgB5do8fE1REhQPVE4vxE9KRs5sjcNDv16SGbC"
    }
  ]
}
```

```bash
# Sweep every configured derivation template at once
./dist/all_address_types -n 1

# ...or point it at your own mnemonic
./dist/all_address_types 5 "word1 word2 ... word12"
```

```bash
# Check balances of many addresses
./dist/blockstream_balance_loop addresses.txt  

# or
./dist/blockstream_balance_loop <<< 12cbQLTFMXRnSzktFkuoG3eHoMeFtpTu3S

# or
echo 12cbQLTFMXRnSzktFkuoG3eHoMeFtpTu3S | ./dist/blockstream_balance_loop

```
```bash
# To get a transaction ID (txid) (address used to send BTC to Hal Finney)
./dist/fetch_utxos 12cbQLTFMXRnSzktFkuoG3eHoMeFtpTu3S
# Example output includes txid field for each UTXO:
[
  {
    "txid": "3832f861eb0fd967fd079da2ee90e415d295dbc81bfb895b73a220aa689c89eb",
    "vout": 0,
    "status": {
      "confirmed": true,
      "block_height": 878308,
      "block_hash": "00000000000000000000287f37f0ddfc5756dddd8eecd2c146d36eafc744fc15",
      "block_time": 1736310913
    },
    "value": 1000,
    "address": null,
    "script_pubkey": null
  }, ...
  
```
```bash
# Look up transaction details by txid
./dist/blockstream_tx 3832f861eb0fd967fd079da2ee90e415d295dbc81bfb895b73a220aa689c89eb

```
```bash
# Create unsigned transaction (JSON from command-line)
./dist/create_tx  '{
  "inputs": [
    {
      "txid": "3832f861eb0fd967fd079da2ee90e415d295dbc81bfb895b73a220aa689c89eb",
      "vout": 0
    }
  ],
  "outputs": [
    {
      "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
      "amount": 100
    }
  ]
}'

# unsigned transaction output example
0100000001eb899c68aa20a2735b89fb1bc8db95d215e490eea29d07fd67d90feb61f832380000000000ffffffff0164000000000000001976a91462e907b15cbf27d5425399ebf6f0fb50ebb88f1888ac00000000

```
```bash
# Sign transaction (JSON from command-line) (WIP don't use)
./dist/sign_tx '{"unsigned_tx_hex":"...","inputs":[{"private_key_wif":"5K...","address":"1A1z..."}]}'


## Advanced Usage Examples

> The generation tools emit JSON, so the pipeline examples below use [`jq`](https://jqlang.github.io/jq/)
> to pull individual fields out.

**Generate a mnemonic and derive its BIP44 addresses**

```bash
./dist/generate_mnemonic \
  | jq -r .bip39_mnemonic \
  | xargs -I {} ./dist/m_44h_0h_0h_i -n 10 {}
```

**Generate mnemonic -> save it -> generate addresses -> save addresses -> check balances**

```bash
./dist/generate_mnemonic \
  | tee mnemonic.txt \
  | jq -r .bip39_mnemonic \
  | xargs -I {} ./dist/m_44h_0h_0h_i -n 10 "{}" \
  | jq -r '.addresses[].address' \
  > ./addresses.txt \
  && ./dist/blockstream_balance_loop ./addresses.txt
```

**Generate mnemonic -> addresses -> check balances in one pipeline**

```bash
while true; do
    ./dist/generate_mnemonic \
        | tee ./mnemonic.txt \
        | jq -r .bip39_mnemonic \
        | xargs -I {} ./dist/m_44h_0h_0h_i -n 10 "{}" \
        | jq -r '.addresses[].address' \
        > ./addresses.txt \
        && ./dist/blockstream_balance_loop ./addresses.txt \
        | tee /dev/tty \
        | grep -q -v "Balance: 0 satoshis" \
        && break
done
```

**Sweep every derivation template from one mnemonic -> check balances**

```bash
./dist/all_address_types 10 \
  | jq -r '.[].addresses[].address' \
  > ./addresses.txt \
  && ./dist/blockstream_balance_loop ./addresses.txt
```

**Generate addresses from quotes -> extract addresses -> check balances**

```bash
cat ./quotes.txt \
    | while read -r quote; do ./dist/brain_wallet "$quote"; done \
    | jq -r .address \
    > ./addresses.txt \
    && ./dist/blockstream_balance_loop ./addresses.txt
```

**Process multiple passphrases from a file**

```bash
while IFS= read -r line || [ -n "$line" ]; do \
./dist/brain_wallet "$line"; done \
< ./quotes.txt \
| jq -r .address > ./addresses.txt \
&& ./dist/blockstream_balance_loop ./addresses.txt
```

## Tools Overview

Tools are grouped by directory: `gen/` generates keys and addresses, `tx/`
builds and signs transactions, `net/` queries the Blockstream API, and
`src/lib` holds the shared Bitcoin library.

### gen/ — Key and address generation

#### Generate Mnemonic (`gen/generate_mnemonic/`)

- Generates BIP39 mnemonic phrases with the built-in English wordlist
- `-s/--strength` selects 128, 160, 192, 224 or 256 bits (12/15/18/21/24 words)
- Outputs the mnemonic, the BIP39 seed and the BIP32 root key as JSON

#### Brain Wallet (`gen/brain_wallet/`)

- Generates a Bitcoin wallet from a passphrase
- Uses SHA-256 to derive the private key from the passphrase
- Uses **uncompressed** public keys and WIF (both start with `5`)
- **Insecure by design — never use for real funds**

#### All Address Types (`gen/all_address_types/`)

- Walks every configured derivation template in one pass
- Covers P2PKH, P2SH-P2WPKH, P2WPKH and P2TR across pre-HD, Electrum,
  MultiBit, Bitcoin Core, BIP44, BIP49, BIP84 and BIP86 styles
- Emits a JSON array grouped by template

#### The `m-...` derivation tools (`gen/m_*/`)

One tool per historical derivation scheme. They differ only in the path walked,
the address scheme and where the account extended keys come from, so each is a
thin wrapper around a shared `DerivationSpec` in `gen/lib`.

| Tool | Derivation Path | Address Type | Era / Wallet |
|---|---|---|---|
| `m_0_i` | `m/0/{i}` | P2PKH | Pre-HD (2009) |
| `m_0h_i` | `m/0'/{i}` | P2PKH | Electrum (2011) |
| `m_0h_0_0h` | `m/0'/0/0'` | P2PKH | MultiBit HD |
| `m_0h_0_i` | `m/0'/0/{i}` | P2PKH | MultiBit Classic |
| `m_0h_0_ih` | `m/0'/0/{i}'` | P2PKH | MultiBit Classic, hardened index |
| `m_0h_0h_0h` | `m/0'/0'/0'` | P2PKH | Bitcoin Core HD (2012) |
| `m_0h_0h_i` | `m/0'/0'/{i}` | P2PKH | Bitcoin Core HD |
| `m_0h_0h_ih` | `m/0'/0'/{i}'` | P2PKH | Bitcoin Core HD, hardened index |
| `m_44h_0h_0h_0` | `m/44'/0'/0'/0` | P2PKH | BIP44 external chain |
| `m_44h_0h_0h_i` | `m/44'/0'/0'/{i}` | P2PKH | BIP44 raw account |
| `m_44h_0h_0h_ih` | `m/44'/0'/0'/{i}'` | P2PKH | BIP44 raw, hardened index |
| `m_44h_0h_0h_0_i` | `m/44'/0'/0'/0/{i}` | P2PKH | BIP44 receive chain |
| `m_44h_0h_0h_0_ih` | `m/44'/0'/0'/0/{i}'` | P2PKH | BIP44 receive chain, hardened index |
| `m_49h_0h_0h_0_i` | `m/49'/0'/0'/0/{i}` | P2SH-P2WPKH | BIP49 (2016) |
| `m_49h_0h_0h_0_ih` | `m/49'/0'/0'/0/{i}'` | P2SH-P2WPKH | BIP49, hardened index |
| `m_84h_0h_0h_0_i` | `m/84'/0'/0'/0/{i}` | P2WPKH | BIP84 (2017) |
| `m_84h_0h_0h_0_ih` | `m/84'/0'/0'/0/{i}'` | P2WPKH | BIP84, hardened index |
| `m_86h_0h_0h_i` | `m/86'/0'/0'/0/{i}` | P2TR | BIP86 (2021) |

Shared argument syntax (all tools):

```bash
./dist/m_<tool>                       # 1 address from the test mnemonic
./dist/m_<tool> 5                     # 5 addresses from the test mnemonic
./dist/m_<tool> "mnemonic words"      # 1 address from your mnemonic
./dist/m_<tool> "mnemonic words" 5    # 5 addresses from your mnemonic
./dist/m_<tool> -n 5 "mnemonic words" # same, explicit flags
./dist/m_<tool> --help
```

> **Note:** `gen/m-86h-0h-0h-i.py` in the Python project derives
> `m/86'/0'/0'/0/{i}` but prints `m/86'/0'/0'/{i}`. The Rust tool reports the path
> it actually walks. Keys and addresses are identical either way.

#### Shared library (`gen/lib/`)

`btcx_gen` holds everything the binaries share: BIP39 parsing and seed
derivation, BIP32 extended key derivation (including `zpub`/`zprv`), the four
address encoders, WIF encoding, the JSON report types and the CLI parsing.

### tx/ — Transactions

#### Create Transaction (`tx/create_tx/`)

- CLI tool to create unsigned Bitcoin transactions
- Accepts JSON input via command-line argument or stdin
- Accepts inputs (txid, vout) and outputs (address, amount)
- Returns hex-encoded transaction

### Sign Transaction (`tx/sign_tx/`)

- CLI tool to sign unsigned Bitcoin transactions
- Accepts JSON input via command-line argument or stdin
- Takes unsigned transaction hex and private keys (WIF format) with corresponding addresses
- Signs all inputs using ECDSA signatures
- Returns hex-encoded signed transaction ready for broadcast
- Supports P2PKH (Pay-to-Public-Key-Hash) transactions

### Fetch UTXOs (`tx/fetch_utxos/`)

- CLI tool to fetch Unspent Transaction Outputs for a Bitcoin address
- Queries Blockstream API for UTXO information
- Outputs JSON formatted UTXO data including txid, vout, value, and confirmation status

### Estimate Fee (`tx/estimate_fee/`)

- CLI tool to estimate Bitcoin transaction fees
- Queries fee estimation APIs for current network conditions
- Provides fee estimates for different confirmation targets

### net/ — Blockstream API queries

#### Blockstream Balance Loop (`net/blockstream_info/`)

- CLI tool to check Bitcoin address balances
- Reads addresses from a file or stdin (use `-` as filename or pipe input)
- Queries Blockstream API for balance info
- Stops when it finds a non-zero balance
- Supports `--help` flag for usage information

#### Blockstream Transaction Lookup (`net/blockstream_tx/`)

- CLI tool to look up Bitcoin transaction details by txid
- Queries Blockstream API for transaction information
- Displays transaction details: inputs, outputs, fees, confirmation status
- Outputs both human-readable format and JSON

#### Broadcast Transaction (`net/broadcast_tx/`)

- CLI tool to broadcast signed Bitcoin transactions to the network
- Submits transaction hex to Blockstream API
- Returns transaction ID (txid) upon successful broadcast

## Tests

```bash
cargo test --workspace
```

`gen/lib/tests/golden.rs` compares every generation tool against a fixture in
`gen/lib/tests/fixtures/`. Those fixtures were captured from the equivalent
`btc-py-scripts/gen/*.py` scripts, so the tests assert the Rust ports reproduce
the Python implementation exactly.

## Dependencies

### Key Dependencies

- `secp256k1` - Elliptic curve cryptography for Bitcoin
- `bitcoin` - Bitcoin library (addresses, BIP32 extended keys, WIF, SegWit/Taproot)
- `bip39` - BIP39 mnemonic parsing, generation and seed derivation
- `serde` & `serde_json` - Serialization
- `thiserror` - Error type definitions

### Development

- **Dependency Management:** Automated via Dependabot for Cargo packages
- **Dependencies:** All Rust crates defined in respective `Cargo.toml` files
- **Language:** Rust (systems programming language, memory safe)

## Probability Note

This project is primarily for educational purposes. The odds of finding an address with some bitcoins in it per cycle are approximately 2.94×10⁻³¹ to 1, assuming there are about 100 million seeds in use. This probability is extremely low, reflecting the vastness of the seed space and the security of Bitcoin's design. Alternatively, as a ratio, the odds are 1 to 3.4×10³⁰.

## License

[Add your license information here]

## Contributing

[Add contribution guidelines here]
