//! Host tests for the `base58` module (slice 2 of palinurus-core).
//!
//! Verifies the `Pubkey` newtype round-trips bytes ↔ base58, exposes the program
//! ids our plugins need, and errors on bad input. The known base58 strings below
//! are ground truth from `@solana/web3.js` + `@solana/spl-memo` (captured
//! 2026-07-19 via `tools/`); the system-program id is additionally cross-checked
//! against the `solana_program` oracle dev-dependency.

use palinurus_core::base58::{Base58Error, Pubkey};
use std::str::FromStr;

/// Known base58 + byte vectors (ground truth from @solana/web3.js + @solana/spl-memo).
struct Vector { name: &'static str, b58: &'static str, bytes: [u8; 32] }

const VECTORS: &[Vector] = &[
    Vector { name: "system",  b58: "11111111111111111111111111111111",
        bytes: [0x00; 32] },
    Vector { name: "memo_v3", b58: "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        bytes: [0x05,0x4a,0x53,0x5a,0x99,0x29,0x21,0x06,0x4d,0x24,0xe8,0x71,0x60,0xda,0x38,0x7c,
                0x7c,0x35,0xb5,0xdd,0xbc,0x92,0xbb,0x81,0xe4,0x1f,0xa8,0x40,0x41,0x05,0x44,0x8d] },
    Vector { name: "sas",     b58: "22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG",
        bytes: [0x0f,0x5e,0x9e,0xd5,0x37,0x1e,0x2c,0x70,0x89,0x8c,0xa9,0xfd,0x0e,0x77,0xc0,0x06,
                0x5c,0xab,0x5d,0xa0,0x2e,0x56,0x67,0x8b,0x27,0x13,0x38,0x2a,0xf3,0x74,0x59,0xb7] },
    Vector { name: "test_42", b58: "5TeWSsjg2gbxCyWVniXeCmwM7UtHTCK7svzJr5xYJzHf",
        bytes: [0x42; 32] },
];

#[test]
fn roundtrip_bytes_to_string_and_back() {
    for v in VECTORS {
        let pk = Pubkey::from_bytes(v.bytes);
        let s = pk.to_string();
        assert_eq!(s, v.b58, "[{}] to_string must match known base58", v.name);
        let back = Pubkey::from_str(v.b58).expect("[{}] from_str must parse");
        assert_eq!(back.to_bytes(), v.bytes, "[{}] from_str must round-trip bytes", v.name);
    }
}

#[test]
fn display_impl_matches_to_string() {
    for v in VECTORS {
        let pk = Pubkey::from_bytes(v.bytes);
        assert_eq!(format!("{pk}"), v.b58, "[{}] Display must equal base58", v.name);
    }
}

#[test]
fn from_str_trait_parses() {
    // `FromStr` trait impl (callable via `Pubkey::from_str` with the trait in scope,
    // imported at crate root of this test file).
    let pk = Pubkey::from_str(VECTORS[1].b58).unwrap();
    assert_eq!(pk.to_bytes(), VECTORS[1].bytes);
}

#[test]
fn constants_match_known_program_ids() {
    assert_eq!(Pubkey::SYSTEM.to_string(), "11111111111111111111111111111111");
    assert_eq!(Pubkey::MEMO.to_string(),  "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
    assert_eq!(Pubkey::SAS.to_string(),   "22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG");
    // Bytes too (guards against a swapped-bytes constant).
    assert_eq!(Pubkey::SYSTEM.to_bytes(), [0x00; 32]);
    assert_eq!(Pubkey::MEMO.to_bytes(),  VECTORS[1].bytes);
    assert_eq!(Pubkey::SAS.to_bytes(),   VECTORS[2].bytes);
}

#[test]
fn from_str_rejects_wrong_length() {
    // A valid base58 string that decodes to != 32 bytes (e.g. 31 or 33).
    // 33 zero-bytes base58 != system; use a known 31-byte pubkey-ish string.
    // "1" decodes to a single zero byte -> wrong length.
    assert_eq!(Pubkey::from_str("1"), Err(Base58Error::InvalidLength));
    // Two chars decoding to 2 bytes.
    assert_eq!(Pubkey::from_str("12"), Err(Base58Error::InvalidLength));
}

#[test]
fn from_str_rejects_invalid_char() {
    // base58 (Bitcoin alphabet) excludes 0, O, I, l. '0' is invalid.
    // Build a 32-byte-looking string containing an invalid char.
    assert_eq!(Pubkey::from_str("0").err(), Some(Base58Error::InvalidChar));
    assert_eq!(Pubkey::from_str("O").err(), Some(Base58Error::InvalidChar));
    assert_eq!(Pubkey::from_str("I").err(), Some(Base58Error::InvalidChar));
    assert_eq!(Pubkey::from_str("l").err(), Some(Base58Error::InvalidChar));
}

#[test]
fn from_str_rejects_empty() {
    assert_eq!(Pubkey::from_str(""), Err(Base58Error::Empty));
}

#[test]
fn as_bytes_returns_inner_ref() {
    let pk = Pubkey::from_bytes(VECTORS[3].bytes);
    assert_eq!(pk.as_bytes(), &VECTORS[3].bytes);
}

#[test]
fn pubkey_is_copy_clone_eq() {
    let a = Pubkey::SYSTEM;
    let b = a; // Copy
    assert_eq!(a, b); // PartialEq
    let c = Pubkey::MEMO;
    assert_ne!(a, c);
}

// ---- Oracle: cross-check the system-program constant against solana_program ----
#[cfg(test)]
mod solana_oracle {
    use super::*;

    #[test]
    fn system_constant_matches_solana_program() {
        let ref_id = solana_program::system_program::id();
        let ref_b58 = bs58::encode(ref_id.to_bytes()).into_string();
        assert_eq!(Pubkey::SYSTEM.to_string(), ref_b58, "SYSTEM must match solana_program::system_program::id()");
        assert_eq!(Pubkey::SYSTEM.to_bytes(), ref_id.to_bytes());
    }
}