//! Golden file tests for the `gen/` tools.
//!
//! Every fixture in `tests/fixtures/` was captured from the matching
//! `btc-py-scripts/gen/*.py` script, so these tests assert that the Rust ports
//! reproduce the Python implementation exactly — keys, addresses, extended keys
//! and all.
//!
//! # One intentional deviation
//!
//! `fixtures/m-86h-0h-0h-i.json` has its `derivation_path` values rewritten.
//! The Python script derives `m/86'/0'/0'/0/{i}` but prints `m/86'/0'/0'/{i}`;
//! the fixture records the path the keys were really produced from.

use std::path::{Path, PathBuf};

use btcx_gen::{brain_wallet, mnemonic, spec, template, ExtendedKey, Result};
use serde_json::Value;

/// The public test mnemonic shared by every fixture.
const TEST_MNEMONIC: &str = mnemonic::DEFAULT_MNEMONIC;

/// The passphrase used by `Brain_wallet.py`'s test.
const BRAIN_WALLET_PASSPHRASE: &str = "testpass";

/// Number of addresses captured in the `m-...` fixtures.
const DERIVATION_COUNT: usize = 3;

fn fixture(name: &str) -> Value {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_str(&contents)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

/// Asserts two serialised documents are equal, reporting the first difference.
fn assert_matches(tool: &str, actual: &Value, expected: &Value) {
    if actual == expected {
        return;
    }
    panic!(
        "{tool} does not match its golden fixture\n  actual:   {actual}\n  expected: {expected}"
    );
}

/// Runs every `m-...` derivation tool and compares it with its fixture.
#[test]
fn derivation_tools_match_the_python_reference() {
    let tools: [(&str, spec::DerivationSpec); 18] = [
        ("m-0-i", spec::M_0_I),
        ("m-0h-i", spec::M_0H_I),
        ("m-0h-0-0h", spec::M_0H_0_0H),
        ("m-0h-0-i", spec::M_0H_0_I),
        ("m-0h-0-ih", spec::M_0H_0_IH),
        ("m-0h-0h-0h", spec::M_0H_0H_0H),
        ("m-0h-0h-i", spec::M_0H_0H_I),
        ("m-0h-0h-ih", spec::M_0H_0H_IH),
        ("m-44h-0h-0h-0", spec::M_44H_0H_0H_0),
        ("m-44h-0h-0h-0-i", spec::M_44H_0H_0H_0_I),
        ("m-44h-0h-0h-0-ih", spec::M_44H_0H_0H_0_IH),
        ("m-44h-0h-0h-i", spec::M_44H_0H_0H_I),
        ("m-44h-0h-0h-ih", spec::M_44H_0H_0H_IH),
        ("m-49h-0h-0h-0-i", spec::M_49H_0H_0H_0_I),
        ("m-49h-0h-0h-0-ih", spec::M_49H_0H_0H_0_IH),
        ("m-84h-0h-0h-0-i", spec::M_84H_0H_0H_0_I),
        ("m-84h-0h-0h-0-ih", spec::M_84H_0H_0H_0_IH),
        ("m-86h-0h-0h-i", spec::M_86H_0H_0H_I),
    ];

    for (name, spec) in tools {
        let report = spec
            .run(TEST_MNEMONIC, "", DERIVATION_COUNT)
            .unwrap_or_else(|error| panic!("{name} failed to derive: {error}"));
        let actual = serde_json::to_value(&report).expect("serialises");
        assert_matches(name, &actual, &fixture(&format!("{name}.json")));
    }
}

/// Runs the template walk and compares it with the Python fixture.
#[test]
fn all_address_types_matches_the_python_reference() {
    let groups = all_types(1).expect("derives");
    let actual = serde_json::to_value(&groups).expect("serialises");

    assert_matches(
        "All_address_types",
        &actual,
        &fixture("All_address_types.json"),
    );
}

/// Runs the brain wallet derivation and compares it with the Python fixture.
#[test]
fn brain_wallet_matches_the_python_reference() {
    let wallet = brain_wallet::derive(BRAIN_WALLET_PASSPHRASE).expect("derives");
    let actual = serde_json::to_value(&wallet).expect("serialises");

    assert_matches("Brain_wallet", &actual, &fixture("Brain_wallet.json"));
}

/// Checks that an invalid mnemonic is rejected rather than silently derived.
#[test]
fn invalid_mnemonics_are_rejected() {
    let outcome = spec::M_44H_0H_0H_I.run("not a real mnemonic phrase", "", 1);

    assert!(
        outcome.is_err(),
        "expected an invalid mnemonic to be rejected"
    );
}

/// Helper mirroring the `all_address_types` binary's logic.
fn all_types(count: usize) -> Result<Vec<btcx_gen::TemplateGroup>> {
    let seed = mnemonic::seed(TEST_MNEMONIC, "")?;
    let master = ExtendedKey::master(&seed)?;

    template::build_groups(&master, count)
}
