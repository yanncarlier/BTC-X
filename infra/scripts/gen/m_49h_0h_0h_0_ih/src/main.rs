//! Rust port of `gen/m-49h-0h-0h-0-ih.py`.
//!
//! BIP49 P2SH-wrapped P2WPKH addresses with a hardened receive index.

use btcx_gen::{cli, spec};

/// One line summary printed by `--help`.
const DESCRIPTION: &str = "Generate BIP49 external chain hardened index m/49'/0'/0'/0/{i}'.";

/// The derivation programme this tool implements.
const SPEC: spec::DerivationSpec = spec::M_49H_0H_0H_0_IH;

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
