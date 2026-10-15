//! Rust port of `gen/Brain_wallet.py`.
//!
//! # Warning
//! Brain wallets are trivially brute forced. This exists to show how the
//! construction works; never use one for real funds.

use btcx_gen::{brain_wallet, cli};

fn main() {
    let passphrase = cli::join_passphrase(&cli::argv());

    cli::print_json(brain_wallet::derive(&passphrase), 4);
}
