//! BIP32 extended key helpers.
//!
//! The tools in this workspace need to print extended keys with different
//! SLIP-132 version prefixes (`xprv`/`xpub` for legacy accounts, `zprv`/`zpub`
//! for BIP84 accounts), so this module derives keys with `bitcoin::bip32` and
//! serialises them itself.

use bitcoin::bip32::{ChildNumber, Xpriv, Xpub};
use bitcoin::hashes::{sha256, Hash};
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::PublicKey;

use crate::error::{GenError, Result};

/// BIP32 derivation offset applied to an index to mark it as hardened.
pub const HARDENED_OFFSET: u32 = 0x8000_0000;

/// The four-byte version prefixes used when serialising an extended key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyVersion {
    /// `xprv` / `xpub` — the standard Bitcoin mainnet prefixes.
    Xpub,
    /// `zprv` / `zpub` — the SLIP-132 prefixes used for native SegWit accounts.
    Zpub,
}

impl KeyVersion {
    /// Version bytes prefixed to a serialised *private* extended key.
    const fn private_bytes(self) -> [u8; 4] {
        match self {
            Self::Xpub => [0x04, 0x88, 0xAD, 0xE4],
            Self::Zpub => [0x04, 0xB2, 0x43, 0x0C],
        }
    }

    /// Version bytes prefixed to a serialised *public* extended key.
    const fn public_bytes(self) -> [u8; 4] {
        match self {
            Self::Xpub => [0x04, 0x88, 0xB2, 0x1E],
            Self::Zpub => [0x04, 0xB2, 0x47, 0x46],
        }
    }
}

/// A BIP32 extended private key on the Bitcoin mainnet.
#[derive(Debug, Clone)]
pub struct ExtendedKey {
    inner: Xpriv,
}

impl ExtendedKey {
    /// Builds the BIP32 master key for `seed` using the mainnet version bytes.
    ///
    /// # Errors
    /// Returns [`GenError::Derivation`] when the seed is not a valid BIP32 seed
    /// (16..=64 bytes is required by the specification).
    pub fn master(seed: &[u8]) -> Result<Self> {
        let inner = Xpriv::new_master(bitcoin::Network::Bitcoin, seed)
            .map_err(|e| GenError::Derivation(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Derives the child key at `index`, hardened when `hardened` is set.
    ///
    /// # Errors
    /// Returns [`GenError::Derivation`] when the child index is out of range or
    /// the resulting key is invalid.
    pub fn derive_child(&self, index: u32, hardened: bool) -> Result<Self> {
        let raw = if hardened {
            index + HARDENED_OFFSET
        } else {
            index
        };
        let child_number = ChildNumber::from(raw);
        let secp = Secp256k1::new();
        let inner = self
            .inner
            .derive_priv(&secp, &child_number)
            .map_err(|e| GenError::Derivation(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Derives every child of `path` in order, returning the final node.
    ///
    /// # Errors
    /// Propagates any failure from [`ExtendedKey::derive_child`].
    pub fn derive_path(&self, path: &[PathElement]) -> Result<Self> {
        let mut current = self.clone();
        for element in path {
            let (index, hardened) = element.resolve(None)?;
            current = current.derive_child(index, hardened)?;
        }
        Ok(current)
    }

    /// Returns the secret key held by this node.
    ///
    /// # Errors
    /// Returns [`GenError::InvalidKey`] if the serialised key is not 32 bytes.
    pub fn secret_key(&self) -> Result<SecretKey> {
        SecretKey::from_slice(&self.encode_key(true)[46..])
            .map_err(|e| GenError::InvalidKey(e.to_string()))
    }

    /// Returns the compressed public key held by this node.
    ///
    /// # Errors
    /// Returns [`GenError::InvalidKey`] if the encoded key cannot be parsed.
    pub fn public_key(&self) -> Result<PublicKey> {
        let mut compressed = [0u8; 33];
        compressed.copy_from_slice(&self.encode_key(false)[45..]);
        let inner = bitcoin::secp256k1::PublicKey::from_slice(&compressed)
            .map_err(|e| GenError::InvalidKey(e.to_string()))?;
        Ok(bitcoin::PublicKey::new(inner))
    }

    /// Serialises this node as an extended *private* key (e.g. `xprv...`).
    pub fn to_extended_private(&self, version: KeyVersion) -> String {
        self.encode_with_version(version.private_bytes(), true)
    }

    /// Serialises this node as an extended *public* key (e.g. `xpub...`).
    pub fn to_extended_public(&self, version: KeyVersion) -> String {
        self.encode_with_version(version.public_bytes(), false)
    }

    /// The BIP32 78-byte serialisation, extended to an `xprv` or `xpub`.
    fn encode_key(&self, private: bool) -> [u8; 78] {
        if private {
            self.inner.encode()
        } else {
            Xpub::from_priv(&Secp256k1::new(), &self.inner).encode()
        }
    }

    fn encode_with_version(&self, version: [u8; 4], private: bool) -> String {
        let mut payload = self.encode_key(private);
        payload[0..4].copy_from_slice(&version);
        base58_check(&payload)
    }
}

/// A single element of a BIP32 derivation path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathElement {
    /// A fixed, non-hardened child index.
    Fixed(u32),
    /// A fixed, hardened child index.
    FixedHardened(u32),
    /// The address index supplied by the caller, non-hardened.
    Index,
    /// The address index supplied by the caller, hardened.
    IndexHardened,
}

impl PathElement {
    /// Resolves this element to a `(index, hardened)` pair.
    ///
    /// `index` is only consulted for [`PathElement::Index`] and
    /// [`PathElement::IndexHardened`].
    ///
    /// # Errors
    /// Returns [`GenError::Derivation`] when an [`PathElement::Index`] variant is
    /// resolved without a caller supplied index.
    pub fn resolve(&self, index: Option<u32>) -> Result<(u32, bool)> {
        match self {
            Self::Fixed(value) => Ok((*value, false)),
            Self::FixedHardened(value) => Ok((*value, true)),
            Self::Index => index.map(|value| (value, false)).ok_or_else(|| {
                GenError::Derivation("address index requested but not supplied".to_string())
            }),
            Self::IndexHardened => index.map(|value| (value, true)).ok_or_else(|| {
                GenError::Derivation("address index requested but not supplied".to_string())
            }),
        }
    }

    /// Renders the element as it appears in a human readable derivation path.
    ///
    /// `index` is only used for the [`PathElement::Index`] variants.
    pub fn render(&self, index: Option<u32>) -> String {
        match self {
            Self::Fixed(value) => value.to_string(),
            Self::FixedHardened(value) => format!("{value}'"),
            Self::Index => index.map(|value| value.to_string()).unwrap_or_default(),
            Self::IndexHardened => index.map(|value| format!("{value}'")).unwrap_or_default(),
        }
    }
}

/// Renders a full derivation path such as `m/44'/0'/0'/0/1`.
pub fn render_path(path: &[PathElement], index: Option<u32>) -> String {
    let mut rendered = String::from("m");
    for element in path {
        rendered.push('/');
        rendered.push_str(&element.render(index));
    }
    rendered
}

/// Applies double-SHA256 checksum and Base58 encoding to `payload`.
fn base58_check(payload: &[u8]) -> String {
    let first = sha256::Hash::hash(payload);
    let second = sha256::Hash::hash(first.as_byte_array());
    let mut full = payload.to_vec();
    full.extend_from_slice(&second.as_byte_array()[..4]);
    bs58::encode(full).into_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The BIP32 test vector 1 seed from the specification.
    const SEED: [u8; 16] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
        0x0F,
    ];

    #[test]
    fn master_key_matches_bip32_test_vector_1() {
        let master = ExtendedKey::master(&SEED).expect("valid seed");

        assert_eq!(
            master.to_extended_private(KeyVersion::Xpub),
            "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi"
        );
        assert_eq!(
            master.to_extended_public(KeyVersion::Xpub),
            "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8"
        );
    }

    #[test]
    fn derives_hardened_child_from_test_vector_1() {
        let master = ExtendedKey::master(&SEED).expect("valid seed");
        let child = master.derive_child(0, true).expect("valid child");

        assert_eq!(
            child.to_extended_private(KeyVersion::Xpub),
            "xprv9uHRZZhk6KAJC1avXpDAp4MDc3sQKNxDiPvvkX8Br5ngLNv1TxvUxt4cV1rGL5hj6KCesnDYUhd7oWgT11eZG7XnxHrnYeSvkzY7d2bhkJ7"
        );
    }

    #[test]
    fn zpub_prefix_is_used_when_requested() {
        let master = ExtendedKey::master(&SEED).expect("valid seed");

        assert!(master
            .to_extended_public(KeyVersion::Zpub)
            .starts_with("zpub"));
        assert!(master
            .to_extended_private(KeyVersion::Zpub)
            .starts_with("zprv"));
    }

    #[test]
    fn render_path_uses_prime_marks_for_hardened_elements() {
        let path = [
            PathElement::FixedHardened(44),
            PathElement::FixedHardened(0),
            PathElement::Index,
        ];

        assert_eq!(render_path(&path, Some(3)), "m/44'/0'/3");
    }
}
