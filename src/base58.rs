//! Base58 `Pubkey` newtype — the typed 32-byte Solana address used across the crate.
//!
//! A thin wrapper over `bs58` so the rest of `palinurus-core` and the Palinurus
//! plugins work with typed addresses (`Pubkey`) rather than raw `[u8; 32]` arrays.
//! No new cryptography: base58 is Bitcoin-alphabet encoding, exactly as Solana
//! uses it for `Pubkey::to_string` / `Pubkey::from_str`.
//!
//! Constants:
//! - [`Pubkey::SYSTEM`] — the System program (`111…111`).
//! - [`Pubkey::MEMO`]   — the SPL Memo program v3 (`MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr`).
//! - [`Pubkey::SAS`]    — the Solana Attestation Service (`22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG`).

use std::fmt;
use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};

/// A Solana public key / program id / PDA — 32 bytes, base58-encoded on the wire.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, BorshSerialize, BorshDeserialize)]
pub struct Pubkey([u8; 32]);

/// Errors from base58 decoding a string into a [`Pubkey`].
#[derive(Debug, PartialEq, Eq)]
pub enum Base58Error {
    /// The input string was empty.
    Empty,
    /// The input contained a character outside the Bitcoin base58 alphabet
    /// (excludes `0`, `O`, `I`, `l`).
    InvalidChar,
    /// The input decoded to a byte stream that is not exactly 32 bytes long.
    InvalidLength,
}

impl Pubkey {
    /// Construct from raw 32 bytes (e.g. a PDA hash output or an account owner).
    pub const fn from_bytes(b: [u8; 32]) -> Self {
        Self(b)
    }

    /// The raw 32 bytes.
    pub const fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    /// A reference to the inner 32 bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// The System program (`11111111111111111111111111111111`).
    pub const SYSTEM: Self = Self([0x00; 32]);

    /// The SPL Memo program v3 (`MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr`).
    /// Bytes captured from `@solana/spl-memo` `MEMO_PROGRAM_ID` (2026-07-19).
    pub const MEMO: Self = Self([
        0x05, 0x4a, 0x53, 0x5a, 0x99, 0x29, 0x21, 0x06, 0x4d, 0x24, 0xe8, 0x71,
        0x60, 0xda, 0x38, 0x7c, 0x7c, 0x35, 0xb5, 0xdd, 0xbc, 0x92, 0xbb, 0x81,
        0xe4, 0x1f, 0xa8, 0x40, 0x41, 0x05, 0x44, 0x8d,
    ]);

    /// The Solana Attestation Service program
    /// (`22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG`).
    /// Bytes captured from `@solana/web3.js` (2026-07-19).
    pub const SAS: Self = Self([
        0x0f, 0x5e, 0x9e, 0xd5, 0x37, 0x1e, 0x2c, 0x70, 0x89, 0x8c, 0xa9, 0xfd,
        0x0e, 0x77, 0xc0, 0x06, 0x5c, 0xab, 0x5d, 0xa0, 0x2e, 0x56, 0x67, 0x8b,
        0x27, 0x13, 0x38, 0x2a, 0xf3, 0x74, 0x59, 0xb7,
    ]);
}

impl fmt::Display for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // bs58 encode with the default Bitcoin alphabet (Solana's).
        f.write_str(&bs58::encode(self.0).into_string())
    }
}

impl fmt::Debug for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Debug as the base58 string (readable) tagged `Pubkey(<b58>)`.
        f.debug_tuple("Pubkey").field(&bs58::encode(self.0).into_string()).finish()
    }
}

impl FromStr for Pubkey {
    type Err = Base58Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Base58Error::Empty);
        }
        // bs58 only fails to decode on invalid characters (length is unconstrained
        // — it decodes whatever bytes result). The 32-byte length check follows.
        let bytes = bs58::decode(s).into_vec().map_err(|_| Base58Error::InvalidChar)?;
        if bytes.len() != 32 {
            return Err(Base58Error::InvalidLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }
}

impl From<[u8; 32]> for Pubkey {
    fn from(b: [u8; 32]) -> Self {
        Self(b)
    }
}

impl From<Pubkey> for [u8; 32] {
    fn from(pk: Pubkey) -> Self {
        pk.0
    }
}

impl AsRef<[u8; 32]> for Pubkey {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_constant_is_all_zero() {
        assert_eq!(Pubkey::SYSTEM.to_bytes(), [0x00; 32]);
        assert_eq!(Pubkey::SYSTEM.to_string(), "11111111111111111111111111111111");
    }
}