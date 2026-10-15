//! Rust port of `gen/m-0h-0h-ih.py`.
//!
//! Bitcoin Core pre-BIP44 HD P2PKH addresses with a hardened receive index.

use btcx_gen::{cli, spec};

/// One line summary printed by `--help`.
const DESCRIPTION: &str =
    "Generate Bitcoin Core HD P2PKH (hardened index at m/0'/0'/i') from BIP39 mnemonic.";

/// The derivation programme this tool implements.
const SPEC: spec::DerivationSpec = spec::M_0H_0H_IH;

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
