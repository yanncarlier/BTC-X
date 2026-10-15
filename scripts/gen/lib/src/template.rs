//! Template driven derivation for the `all_address_types` tool.
//!
//! Templates use a compact `m-44h-0h-0h-0-i` notation where `h` marks a
//! hardened element, `i` is the address index and `ih` is a hardened address
//! index. This mirrors the notation used by the Python script of the same name.

use crate::address::{wif_compressed, AddressScheme};
use crate::error::{GenError, Result};
use crate::keys::ExtendedKey;
use crate::report::{TemplateAddress, TemplateGroup};

/// Every template the tool walks, in output order.
pub const TEMPLATES: [&str; 19] = [
    "m",
    "m-0-i",
    "m-0h-i",
    "m-0h-0-i",
    "m-0h-0-ih",
    "m-0h-0-0h",
    "m-0h-0h-i",
    "m-0h-0h-ih",
    "m-0h-0h-0h",
    "m-44h-0h-0h-0",
    "m-44h-0h-0h-0-i",
    "m-44h-0h-0h-0-ih",
    "m-44h-0h-0h-i",
    "m-44h-0h-0h-ih",
    "m-49h-0h-0h-0-i",
    "m-49h-0h-0h-0-ih",
    "m-84h-0h-0h-0-i",
    "m-84h-0h-0h-0-ih",
    "m-86h-0h-0h-i",
];

/// A single parsed template element: the raw value plus its hardened flag.
type Element = (String, bool);

/// Turns `m-44h-0h-0h-0-ih` into its individual path elements.
///
/// `i` and `ih` are kept verbatim as the address index placeholder; every other
/// element is stored as a decimal string with its hardened flag applied.
///
/// # Errors
/// Returns [`GenError::Usage`] when an element is neither a number nor `i`/`ih`.
pub fn parse_template(template: &str) -> Result<Vec<Element>> {
    let mut elements = Vec::new();
    for token in template.split('-').skip(1) {
        let element = match token {
            "i" => ("i".to_string(), false),
            "ih" => ("i".to_string(), true),
            other if other.ends_with('h') && other[..other.len() - 1].parse::<u32>().is_ok() => {
                (other[..other.len() - 1].to_string(), true)
            }
            other if other.parse::<u32>().is_ok() => (other.to_string(), false),
            other => {
                return Err(GenError::Usage(format!(
                    "unsupported template element '{other}' in '{template}'"
                )));
            }
        };
        elements.push(element);
    }
    Ok(elements)
}

/// Picks the address scheme from the template's purpose (its first element).
pub fn scheme_for(template: &str) -> AddressScheme {
    match template.split('-').nth(1) {
        Some("49h") => AddressScheme::P2shP2wpkh,
        Some("84h") => AddressScheme::P2wpkh,
        Some("86h") => AddressScheme::P2tr,
        _ => AddressScheme::P2pkh,
    }
}

/// Renders a template as a human readable derivation path, for example
/// `m/44'/0'/0'/0/i'`.
pub fn pattern_for(template: &str) -> String {
    template
        .replace('-', "/")
        .replace("ih", "i'")
        .replace('h', "'")
}

/// Builds the output group for a single template.
///
/// Templates that contain an address index produce `count` entries; the others
/// produce exactly one entry with a `null` index.
///
/// # Errors
/// Returns an error when a BIP32 derivation step fails.
pub fn build_group(master: &ExtendedKey, template: &str, count: usize) -> Result<TemplateGroup> {
    let elements = parse_template(template)?;
    let scheme = scheme_for(template);
    let has_index = elements.iter().any(|(value, _)| value == "i");

    let mut addresses = Vec::new();
    for index in 0..count.max(1) {
        let index = if has_index { Some(index as u32) } else { None };
        let (node, path) = derive_node(master, &elements, index)?;
        let secret = node.secret_key()?;
        let public = node.public_key()?;

        addresses.push(TemplateAddress {
            index,
            derivation_path: path,
            address_type: scheme.name().to_string(),
            address: scheme.encode(&public)?.to_string(),
            public_key: hex::encode(public.inner.serialize()),
            private_key: hex::encode(secret.secret_bytes()),
            wif: wif_compressed(&secret)?,
        });
        if !has_index {
            break;
        }
    }

    Ok(TemplateGroup {
        template: template.to_string(),
        address_type: scheme.name().to_string(),
        pattern: pattern_for(template),
        addresses,
    })
}

/// Walks `elements` from `master`, substituting `index` wherever `i` appears.
fn derive_node(
    master: &ExtendedKey,
    elements: &[Element],
    index: Option<u32>,
) -> Result<(ExtendedKey, String)> {
    let mut node = master.clone();
    let mut path = String::from("m");

    for (value, hardened) in elements {
        let child = if value == "i" {
            index.ok_or_else(|| {
                GenError::Usage("indexed template requires an address index".to_string())
            })?
        } else {
            value
                .parse::<u32>()
                .map_err(|_| GenError::Usage(format!("unsupported template element '{value}'")))?
        };

        node = node.derive_child(child, *hardened)?;
        path.push('/');
        path.push_str(&child.to_string());
        if *hardened {
            path.push('\'');
        }
    }

    Ok((node, path))
}

/// Builds every configured template group in [`TEMPLATES`] order.
///
/// # Errors
/// Returns an error when a BIP32 derivation step fails.
pub fn build_groups(master: &ExtendedKey, count: usize) -> Result<Vec<TemplateGroup>> {
    TEMPLATES
        .iter()
        .map(|template| build_group(master, template, count))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hardened_and_index_elements() {
        let elements = parse_template("m-44h-0h-0h-0-ih").expect("parses");

        assert_eq!(
            elements,
            vec![
                ("44".to_string(), true),
                ("0".to_string(), true),
                ("0".to_string(), true),
                ("0".to_string(), false),
                ("i".to_string(), true),
            ]
        );
    }

    #[test]
    fn master_template_has_no_elements() {
        assert!(parse_template("m").expect("parses").is_empty());
    }

    #[test]
    fn renders_patterns_with_prime_marks() {
        assert_eq!(pattern_for("m-44h-0h-0h-0-ih"), "m/44'/0'/0'/0/i'");
        assert_eq!(pattern_for("m"), "m");
    }

    #[test]
    fn selects_scheme_from_purpose() {
        assert_eq!(scheme_for("m-49h-0h-0h-0-i"), AddressScheme::P2shP2wpkh);
        assert_eq!(scheme_for("m-84h-0h-0h-0-i"), AddressScheme::P2wpkh);
        assert_eq!(scheme_for("m-86h-0h-0h-i"), AddressScheme::P2tr);
        assert_eq!(scheme_for("m-0h-i"), AddressScheme::P2pkh);
        assert_eq!(scheme_for("m"), AddressScheme::P2pkh);
    }
}
