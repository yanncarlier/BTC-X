//! Declarative descriptions of each derivation programme.
//!
//! Every `m-...` tool in this directory differs from the others only in the path
//! it walks, the address scheme it encodes with, and where it reads the account
//! level extended keys from. Describing those differences as data keeps the 18
//! binaries down to a handful of lines each.

use crate::address::{wif_compressed, AddressScheme};
use crate::error::Result;
use crate::keys::{render_path, ExtendedKey, KeyVersion, PathElement};
use crate::mnemonic;
use crate::report::{AddressRecord, DerivationReport};

/// How the trailing address index is hardened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexStyle {
    /// `.../{i}` — the usual BIP32 receive chain.
    NonHardened,
    /// `.../{i}'` — a hardened address index.
    Hardened,
}

impl IndexStyle {
    const fn hardened(self) -> bool {
        matches!(self, Self::Hardened)
    }

    const fn element(self) -> PathElement {
        match self {
            Self::NonHardened => PathElement::Index,
            Self::Hardened => PathElement::IndexHardened,
        }
    }
}

/// Where the account level extended keys come from.
#[derive(Debug, Clone, Copy)]
pub enum AccountSource {
    /// The BIP32 master key itself.
    Master,
    /// An explicit path below the master key.
    AtPath(&'static [PathElement]),
    /// The fixed prefix of the address path.
    Base,
    /// The first derived address node.
    FirstAddress,
}

/// A complete description of one derivation programme.
#[derive(Debug, Clone, Copy)]
pub struct DerivationSpec {
    /// The template name, matching the Python script it ports.
    pub template: &'static str,
    /// The `address_type` string reported in the JSON output.
    pub label: &'static str,
    /// The fixed path elements walked before the address index.
    pub base: &'static [PathElement],
    /// How the address index is applied, or `None` for fixed paths.
    pub index: Option<IndexStyle>,
    /// Where the account extended keys are read from.
    pub account_source: AccountSource,
    /// The address encoding scheme.
    pub scheme: AddressScheme,
    /// The SLIP-132 prefixes used for the account extended keys.
    pub key_version: KeyVersion,
}

impl DerivationSpec {
    /// The full address path, including the index element when present.
    fn address_path(&self) -> Vec<PathElement> {
        let mut path = self.base.to_vec();
        if let Some(style) = self.index {
            path.push(style.element());
        }
        path
    }

    /// Derives `count` addresses from `mnemonic` and assembles the JSON report.
    ///
    /// # Errors
    /// Returns an error when the mnemonic is invalid or a BIP32 derivation step
    /// fails.
    pub fn run(
        &self,
        mnemonic_phrase: &str,
        passphrase: &str,
        count: usize,
    ) -> Result<DerivationReport> {
        let seed = mnemonic::seed(mnemonic_phrase, passphrase)?;
        let master = ExtendedKey::master(&seed)?;
        let base = master.derive_path(self.base)?;

        let mut addresses = Vec::with_capacity(count);
        for index in 0..count as u32 {
            let node = match self.index {
                Some(style) => base.derive_child(index, style.hardened())?,
                None => base.clone(),
            };
            addresses.push(self.record(&node, Some(index))?);
        }

        // Paths for fixed templates carry no index, so render them without one.
        if self.index.is_none() {
            for (position, record) in addresses.iter_mut().enumerate() {
                record.index = Some(position as u32);
                record.derivation_path = render_path(self.base, None);
            }
        }

        let account_node = match self.account_source {
            AccountSource::Master => Some(master),
            AccountSource::AtPath(path) => Some(master.derive_path(path)?),
            AccountSource::Base => Some(base),
            AccountSource::FirstAddress => addresses
                .first()
                .map(|_| match self.index {
                    Some(style) => base.derive_child(0, style.hardened()),
                    None => Ok(base.clone()),
                })
                .transpose()?,
        };

        Ok(DerivationReport {
            mnemonic_phrase: mnemonic_phrase.trim().to_string(),
            passphrase: passphrase.to_string(),
            seed_hex: hex::encode(seed),
            address_type: self.label.to_string(),
            account_extended_public_key: account_node
                .as_ref()
                .map(|node| node.to_extended_public(self.key_version)),
            account_extended_private_key: account_node
                .as_ref()
                .map(|node| node.to_extended_private(self.key_version)),
            addresses,
        })
    }

    fn record(&self, node: &ExtendedKey, index: Option<u32>) -> Result<AddressRecord> {
        let secret = node.secret_key()?;
        let public = node.public_key()?;

        Ok(AddressRecord {
            index,
            derivation_path: render_path(&self.address_path(), index),
            address: self.scheme.encode(&public)?.to_string(),
            public_key: hex::encode(public.inner.serialize()),
            private_key: hex::encode(secret.secret_bytes()),
            wif: wif_compressed(&secret)?,
        })
    }
}

const PURPOSE_44: PathElement = PathElement::FixedHardened(44);
const PURPOSE_49: PathElement = PathElement::FixedHardened(49);
const PURPOSE_84: PathElement = PathElement::FixedHardened(84);
const PURPOSE_86: PathElement = PathElement::FixedHardened(86);
const COIN: PathElement = PathElement::FixedHardened(0);
const ACCOUNT: PathElement = PathElement::FixedHardened(0);
const EXTERNAL: PathElement = PathElement::Fixed(0);

/// Pre-HD style `m/0/{i}`, straight off the master key (2009 Bitcoin Core).
pub const M_0_I: DerivationSpec = DerivationSpec {
    template: "m-0-i",
    label: "pre-HD P2PKH",
    base: &[PathElement::Fixed(0)],
    index: Some(IndexStyle::NonHardened),
    account_source: AccountSource::Master,
    scheme: AddressScheme::P2pkh,
    key_version: KeyVersion::Xpub,
};

/// Electrum style `m/0'/{i}` (2011).
pub const M_0H_I: DerivationSpec = DerivationSpec {
    template: "m-0h-i",
    label: "Electrum-style P2PKH",
    base: &[PathElement::FixedHardened(0)],
    index: Some(IndexStyle::NonHardened),
    account_source: AccountSource::FirstAddress,
    scheme: AddressScheme::P2pkh,
    key_version: KeyVersion::Xpub,
};

/// MultiBit HD style `m/0'/0/0'`.
pub const M_0H_0_0H: DerivationSpec = DerivationSpec {
    template: "m-0h-0-0h",
    label: "MultiBit HD-style P2PKH",
    base: &[
        PathElement::FixedHardened(0),
        PathElement::Fixed(0),
        PathElement::FixedHardened(0),
    ],
    index: None,
    account_source: AccountSource::FirstAddress,
    scheme: AddressScheme::P2pkh,
    key_version: KeyVersion::Xpub,
};

/// MultiBit Classic `m/0'/0/{i}`.
pub const M_0H_0_I: DerivationSpec = DerivationSpec {
    template: "m-0h-0-i",
    label: "MultiBit Classic P2PKH",
    base: &[PathElement::FixedHardened(0), PathElement::Fixed(0)],
    index: Some(IndexStyle::NonHardened),
    account_source: AccountSource::FirstAddress,
    scheme: AddressScheme::P2pkh,
    key_version: KeyVersion::Xpub,
};

/// MultiBit Classic with a hardened address index, `m/0'/0/{i}'`.
pub const M_0H_0_IH: DerivationSpec = DerivationSpec {
    template: "m-0h-0-ih",
    label: "P2PKH hardened index m/0'/0/{i}'",
    base: &[PathElement::FixedHardened(0), PathElement::Fixed(0)],
    index: Some(IndexStyle::Hardened),
    account_source: AccountSource::Base,
    scheme: AddressScheme::P2pkh,
    key_version: KeyVersion::Xpub,
};

/// Bitcoin Core pre-BIP44 HD `m/0'/0'/0'` (2012).
pub const M_0H_0H_0H: DerivationSpec = DerivationSpec {
    template: "m-0h-0h-0h",
    label: "Bitcoin Core-style HD P2PKH",
    base: &[
        PathElement::FixedHardened(0),
        PathElement::FixedHardened(0),
        PathElement::FixedHardened(0),
    ],
    index: None,
    account_source: AccountSource::FirstAddress,
    scheme: AddressScheme::P2pkh,
    key_version: KeyVersion::Xpub,
};

/// Bitcoin Core pre-BIP44 HD with a receive index, `m/0'/0'/{i}`.
pub const M_0H_0H_I: DerivationSpec = DerivationSpec {
    template: "m-0h-0h-i",
    label: "P2PKH base m/0'/0'/i (non-hardened index)",
    base: &[PathElement::FixedHardened(0), PathElement::FixedHardened(0)],
    index: Some(IndexStyle::NonHardened),
    account_source: AccountSource::FirstAddress,
    scheme: AddressScheme::P2pkh,
    key_version: KeyVersion::Xpub,
};

/// Bitcoin Core pre-BIP44 HD with a hardened receive index, `m/0'/0'/{i}'`.
pub const M_0H_0H_IH: DerivationSpec = DerivationSpec {
    template: "m-0h-0h-ih",
    label: "Bitcoin Core-style HD P2PKH (hardened index)",
    base: &[PathElement::FixedHardened(0), PathElement::FixedHardened(0)],
    index: Some(IndexStyle::Hardened),
    account_source: AccountSource::FirstAddress,
    scheme: AddressScheme::P2pkh,
    key_version: KeyVersion::Xpub,
};

/// BIP44 external chain node, `m/44'/0'/0'/0`.
pub const M_44H_0H_0H_0: DerivationSpec = DerivationSpec {
    template: "m-44h-0h-0h-0",
    label: "BIP44 external chain P2PKH",
    base: &[PURPOSE_44, COIN, ACCOUNT, EXTERNAL],
    index: None,
    account_source: AccountSource::FirstAddress,
    scheme: AddressScheme::P2pkh,
    key_version: KeyVersion::Xpub,
};

/// BIP44 external chain receive addresses, `m/44'/0'/0'/0/{i}`.
pub const M_44H_0H_0H_0_I: DerivationSpec = DerivationSpec {
    template: "m-44h-0h-0h-0-i",
    label: "BIP44 external chain index",
    base: &[PURPOSE_44, COIN, ACCOUNT, EXTERNAL],
    index: Some(IndexStyle::NonHardened),
    account_source: AccountSource::Base,
    scheme: AddressScheme::P2pkh,
    key_version: KeyVersion::Xpub,
};

/// BIP44 external chain with a hardened index, `m/44'/0'/0'/0/{i}'`.
pub const M_44H_0H_0H_0_IH: DerivationSpec = DerivationSpec {
    template: "m-44h-0h-0h-0-ih",
    label: "BIP44 external chain hardened index",
    base: &[PURPOSE_44, COIN, ACCOUNT, EXTERNAL],
    index: Some(IndexStyle::Hardened),
    account_source: AccountSource::Base,
    scheme: AddressScheme::P2pkh,
    key_version: KeyVersion::Xpub,
};

/// BIP44 raw account derivation, `m/44'/0'/0'/{i}`.
pub const M_44H_0H_0H_I: DerivationSpec = DerivationSpec {
    template: "m-44h-0h-0h-i",
    label: "BIP44 P2PKH (raw m/44'/0'/0'/{i})",
    base: &[PURPOSE_44, COIN, ACCOUNT],
    index: Some(IndexStyle::NonHardened),
    account_source: AccountSource::Base,
    scheme: AddressScheme::P2pkh,
    key_version: KeyVersion::Xpub,
};

/// BIP44 raw account derivation with a hardened index, `m/44'/0'/0'/{i}'`.
pub const M_44H_0H_0H_IH: DerivationSpec = DerivationSpec {
    template: "m-44h-0h-0h-ih",
    label: "BIP44 P2PKH (hardened index)",
    base: &[PURPOSE_44, COIN, ACCOUNT],
    index: Some(IndexStyle::Hardened),
    account_source: AccountSource::Base,
    scheme: AddressScheme::P2pkh,
    key_version: KeyVersion::Xpub,
};

/// BIP49 P2SH-wrapped P2WPKH receive addresses, `m/49'/0'/0'/0/{i}`.
pub const M_49H_0H_0H_0_I: DerivationSpec = DerivationSpec {
    template: "m-49h-0h-0h-0-i",
    label: "BIP49 external chain index",
    base: &[PURPOSE_49, COIN, ACCOUNT, EXTERNAL],
    index: Some(IndexStyle::NonHardened),
    account_source: AccountSource::Base,
    scheme: AddressScheme::P2shP2wpkh,
    key_version: KeyVersion::Xpub,
};

/// BIP49 P2SH-wrapped P2WPKH with a hardened index, `m/49'/0'/0'/0/{i}'`.
pub const M_49H_0H_0H_0_IH: DerivationSpec = DerivationSpec {
    template: "m-49h-0h-0h-0-ih",
    label: "BIP49 external chain hardened index",
    base: &[PURPOSE_49, COIN, ACCOUNT, EXTERNAL],
    index: Some(IndexStyle::Hardened),
    account_source: AccountSource::Base,
    scheme: AddressScheme::P2shP2wpkh,
    key_version: KeyVersion::Xpub,
};

/// BIP84 native SegWit receive addresses, `m/84'/0'/0'/0/{i}`.
pub const M_84H_0H_0H_0_I: DerivationSpec = DerivationSpec {
    template: "m-84h-0h-0h-0-i",
    label: "BIP84 P2WPKH",
    base: &[PURPOSE_84, COIN, ACCOUNT, EXTERNAL],
    index: Some(IndexStyle::NonHardened),
    account_source: AccountSource::AtPath(&[PURPOSE_84, COIN, ACCOUNT]),
    scheme: AddressScheme::P2wpkh,
    key_version: KeyVersion::Zpub,
};

/// BIP84 native SegWit with a hardened index, `m/84'/0'/0'/0/{i}'`.
pub const M_84H_0H_0H_0_IH: DerivationSpec = DerivationSpec {
    template: "m-84h-0h-0h-0-ih",
    label: "BIP84 P2WPKH hardened index",
    base: &[PURPOSE_84, COIN, ACCOUNT, EXTERNAL],
    index: Some(IndexStyle::Hardened),
    account_source: AccountSource::AtPath(&[PURPOSE_84, COIN, ACCOUNT]),
    scheme: AddressScheme::P2wpkh,
    key_version: KeyVersion::Zpub,
};

/// BIP86 Taproot receive addresses, `m/86'/0'/0'/0/{i}`.
pub const M_86H_0H_0H_I: DerivationSpec = DerivationSpec {
    template: "m-86h-0h-0h-i",
    label: "BIP86 P2TR",
    base: &[PURPOSE_86, COIN, ACCOUNT, EXTERNAL],
    index: Some(IndexStyle::NonHardened),
    account_source: AccountSource::AtPath(&[PURPOSE_86, COIN, ACCOUNT]),
    scheme: AddressScheme::P2tr,
    key_version: KeyVersion::Xpub,
};

/// Every derivation programme, keyed by its template name.
pub const ALL_SPECS: [DerivationSpec; 18] = [
    M_0_I,
    M_0H_I,
    M_0H_0_0H,
    M_0H_0_I,
    M_0H_0_IH,
    M_0H_0H_0H,
    M_0H_0H_I,
    M_0H_0H_IH,
    M_44H_0H_0H_0,
    M_44H_0H_0H_0_I,
    M_44H_0H_0H_0_IH,
    M_44H_0H_0H_I,
    M_44H_0H_0H_IH,
    M_49H_0H_0H_0_I,
    M_49H_0H_0H_0_IH,
    M_84H_0H_0H_0_I,
    M_84H_0H_0H_0_IH,
    M_86H_0H_0H_I,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_template_name_is_unique() {
        let mut names: Vec<&str> = ALL_SPECS.iter().map(|spec| spec.template).collect();
        names.sort_unstable();
        let unique = names.len();
        names.dedup();

        assert_eq!(names.len(), unique, "duplicate template names in ALL_SPECS");
    }

    #[test]
    fn derives_the_bip44_reference_addresses() {
        let report = M_44H_0H_0H_I
            .run(mnemonic::DEFAULT_MNEMONIC, "", 1)
            .expect("derives");

        assert_eq!(report.addresses[0].derivation_path, "m/44'/0'/0'/0");
        assert_eq!(
            report.addresses[0].address,
            "13KE6TffArLh4fVM6uoQzvsYq5vwetJcVM"
        );
        assert_eq!(
            report.account_extended_public_key.as_deref(),
            Some("xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj")
        );
    }

    #[test]
    fn fixed_paths_repeat_the_same_address_for_every_index() {
        let report = M_44H_0H_0H_0
            .run(mnemonic::DEFAULT_MNEMONIC, "", 3)
            .expect("derives");

        assert_eq!(report.addresses.len(), 3);
        assert!(report
            .addresses
            .iter()
            .all(|record| record.address == report.addresses[0].address));
        assert!(report
            .addresses
            .iter()
            .all(|record| record.derivation_path == "m/44'/0'/0'/0"));
        assert_eq!(report.addresses[2].index, Some(2));
    }

    #[test]
    fn bip84_account_keys_use_zprv_prefixes() {
        let report = M_84H_0H_0H_0_I
            .run(mnemonic::DEFAULT_MNEMONIC, "", 1)
            .expect("derives");

        assert!(report
            .account_extended_public_key
            .as_deref()
            .is_some_and(|key| key.starts_with("zpub")));
        assert!(report
            .account_extended_private_key
            .as_deref()
            .is_some_and(|key| key.starts_with("zprv")));
    }

    #[test]
    fn bip86_taproot_addresses_match_the_reference_vector() {
        let report = M_86H_0H_0H_I
            .run(mnemonic::DEFAULT_MNEMONIC, "", 1)
            .expect("derives");

        assert_eq!(report.addresses[0].derivation_path, "m/86'/0'/0'/0/0");
        assert_eq!(
            report.addresses[0].address,
            "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr"
        );
    }
}
