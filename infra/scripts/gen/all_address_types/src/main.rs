//! Rust port of `gen/All_address_types.py`.
//!
//! Walks a fixed list of derivation templates and prints every address type the
//! templates cover, grouped by template. Unlike the `m-...` tools this one also
//! emits *uncompressed* WIFs, matching the Python original.

use btcx_gen::{cli, mnemonic, template, ExtendedKey, Result, TemplateGroup};

/// One line summary printed by `--help`.
const DESCRIPTION: &str = "Generate Bitcoin addresses from a BIP39 mnemonic.";

/// The Python original hard codes an empty BIP39 passphrase.
const BIP39_PASSPHRASE: &str = "";

fn main() {
    let program = cli::program_name();
    let options = cli::resolve(
        &program,
        cli::parse_all_types_args(&program, DESCRIPTION, &cli::argv()),
    );

    cli::print_json(build_groups(&options), 2);
}

/// Derives every configured template from the requested mnemonic.
///
/// # Errors
/// Returns an error when the mnemonic is invalid or a BIP32 derivation step
/// fails.
fn build_groups(options: &cli::DerivationArgs) -> Result<Vec<TemplateGroup>> {
    let seed = mnemonic::seed(&options.mnemonic, BIP39_PASSPHRASE)?;
    let master = ExtendedKey::master(&seed)?;

    template::build_groups(&master, options.count)
}
