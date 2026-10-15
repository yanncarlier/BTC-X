//! Rust port of `gen/m-86h-0h-0h-i.py`.
//!
//! BIP86 Taproot receive addresses.
//!
//! # Difference from the Python original
//!
//! `gen/m-86h-0h-0h-i.py` derives `m/86'/0'/0'/0/{i}` but *reports* the path as
//! `m/86'/0'/0'/{i}`, i.e. it is one level short. This port reports the path it
//! actually walks; every key and address is unchanged.

use btcx_gen::{cli, spec};

/// One line summary printed by `--help`.
const DESCRIPTION: &str = "Generate BIP86 Taproot P2TR addresses from a BIP39 mnemonic.";

/// The derivation programme this tool implements.
const SPEC: spec::DerivationSpec = spec::M_86H_0H_0H_I;

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
