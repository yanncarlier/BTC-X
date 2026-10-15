//! Rust port of `gen/m-0h-0-0h.py`.
//!
//! MultiBit HD style P2PKH addresses.

use btcx_gen::{cli, spec};

/// One line summary printed by `--help`.
const DESCRIPTION: &str = "Generate MultiBit HD-style P2PKH addresses from a BIP39 mnemonic.";

/// The derivation programme this tool implements.
const SPEC: spec::DerivationSpec = spec::M_0H_0_0H;

/// The Python original hard codes an empty BIP39 passphrase.
const BIP39_PASSPHRASE: &str = "";

fn main() {
    let program = cli::program_name();
    let options = cli::resolve(
        &program,
        cli::parse_derivation_args(&program, DESCRIPTION, &cli::argv()),
    );
    let report = SPEC.run(&options.mnemonic, BIP39_PASSPHRASE, options.count);

    cli::print_json(report, 2);
}
