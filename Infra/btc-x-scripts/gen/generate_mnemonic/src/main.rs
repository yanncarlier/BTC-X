//! Rust port of `gen/Generate_mnemonic.py`.
//!
//! Generates a BIP39 mnemonic together with the seed and BIP32 root key derived
//! from it. The English wordlist is built in, so no wordlist file is needed.

use btcx_gen::{cli, mnemonic};

fn main() {
    let strength = cli::resolve(&cli::program_name(), cli::parse_strength_args(&cli::argv()));

    cli::print_json(mnemonic::generate_report(strength), 4);
}
