//! Host tests for the `borsh` module (slice 3 of palinurus-core).
//!
//! Verifies the SAS `create_attestation` instruction DATA serializes byte-for-byte
//! to the canonical sas-lib Codama-generated codec (`tools/verify-borsh.mjs` oracle,
//! run 2026-07-19 against `sas-lib@1.0.10`), and that the memo helper emits raw
//! UTF-8 (memo program v3 takes no discriminator / length prefix).

use palinurus_core::base58::Pubkey;
use palinurus_core::borsh::{memo_ix_data, CreateAttestationIxData};
use borsh::{to_vec, BorshDeserialize};
use std::str::FromStr;

/// Ground-truth bytes from `tools/verify-borsh.mjs` (sas-lib @solana/kit encoder)
/// for: nonce=MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr,
///      data="palinurus:temp=24.7C" (20 bytes), expiry=1753000000 (i64).
const SAS_IX_HEX: &str =
    "06054a535a992921064d24e87160da387c7c35b5ddbc92bb81e41fa8404105448d\
     1400000070616c696e757275733a74656d703d32342e37434\
     0a87c6800000000";

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    hex.chars().filter(|c| !c.is_whitespace()).collect::<String>()
        .as_bytes().chunks(2)
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
        .collect()
}

fn test_vector() -> CreateAttestationIxData {
    CreateAttestationIxData {
        discriminator: 6,
        nonce: Pubkey::from_str("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr").unwrap(),
        data: b"palinurus:temp=24.7C".to_vec(),
        expiry: 1753000000,
    }
}

#[test]
fn sas_create_attestation_matches_sas_lib_codec_byte_for_byte() {
    let ix = test_vector();
    let bytes = to_vec(&ix).expect("borsh serialize");
    let expected = hex_to_bytes(SAS_IX_HEX);
    assert_eq!(bytes, expected, "SAS create_attestation ix DATA must match sas-lib codec byte-for-byte");
}

#[test]
fn sas_ix_layout_is_discriminator_then_nonce_then_len_data_then_expiry() {
    let ix = test_vector();
    let bytes = to_vec(&ix).unwrap();
    assert_eq!(bytes.len(), 65, "total len = 1 + 32 + 4 + 20 + 8");
    assert_eq!(bytes[0], 6, "discriminator u8 = 6");
    let nonce = Pubkey::from_str("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr").unwrap();
    assert_eq!(&bytes[1..33], nonce.as_bytes());
    let len = u32::from_le_bytes(bytes[33..37].try_into().unwrap());
    assert_eq!(len, 20, "data length prefix u32 LE = 20");
    assert_eq!(&bytes[37..57], b"palinurus:temp=24.7C");
    let expiry = i64::from_le_bytes(bytes[57..65].try_into().unwrap());
    assert_eq!(expiry, 1753000000, "expiry i64 LE");
}

#[test]
fn sas_ix_roundtrips_borsh() {
    let ix = test_vector();
    let bytes = to_vec(&ix).unwrap();
    let back = CreateAttestationIxData::try_from_slice(&bytes).expect("borsh deserialize");
    assert_eq!(back.discriminator, 6);
    assert_eq!(back.nonce, ix.nonce);
    assert_eq!(back.data, ix.data);
    assert_eq!(back.expiry, ix.expiry);
}

#[test]
fn sas_ix_different_fields_yield_different_bytes() {
    let a = test_vector();
    let a_bytes = to_vec(&a).unwrap();

    let mut b = a.clone();
    b.expiry = 9999999999;
    assert_ne!(a_bytes, to_vec(&b).unwrap(), "different expiry must yield different bytes");

    let mut c = a.clone();
    c.data = b"palinurus:temp=99.9C".to_vec();
    assert_ne!(a_bytes, to_vec(&c).unwrap(), "different data must yield different bytes");

    let mut d = a.clone();
    d.nonce = Pubkey::SYSTEM;
    assert_ne!(a_bytes, to_vec(&d).unwrap(), "different nonce must yield different bytes");
}

#[test]
fn sas_ix_empty_data_is_valid() {
    // Edge: zero-length attestation payload. Borsh writes u32(0) + no bytes.
    let ix = CreateAttestationIxData {
        discriminator: 6,
        nonce: Pubkey::SYSTEM,
        data: Vec::new(),
        expiry: 0,
    };
    let bytes = to_vec(&ix).unwrap();
    // layout: 1 (disc) + 32 (nonce) + 4 (len prefix) + 0 (empty data) + 8 (expiry) = 45
    assert_eq!(bytes.len(), 1 + 32 + 4 + 8, "empty data: 1+32+4+0+8 = 45");
    let back = CreateAttestationIxData::try_from_slice(&bytes).unwrap();
    assert!(back.data.is_empty());
}

#[test]
fn sas_ix_large_data_length_prefix_is_u32_le() {
    // 256-byte payload → length prefix = 256 as u32 LE = [00, 01, 00, 00].
    let data = vec![0xABu8; 256];
    let ix = CreateAttestationIxData { discriminator: 6, nonce: Pubkey::SYSTEM, data, expiry: 1 };
    let bytes = to_vec(&ix).unwrap();
    let len_bytes = &bytes[33..37];
    assert_eq!(len_bytes, &[0x00, 0x01, 0x00, 0x00], "256 as u32 LE = [00,01,00,00]");
}

#[test]
fn sas_ix_new_constructor_fills_discriminator() {
    let ix = CreateAttestationIxData::new(Pubkey::SYSTEM, vec![1, 2, 3], 42);
    assert_eq!(ix.discriminator, 6);
    assert_eq!(ix.expiry, 42);
    // And it serializes identically to the explicit-discriminator form.
    let explicit = CreateAttestationIxData { discriminator: 6, nonce: Pubkey::SYSTEM, data: vec![1, 2, 3], expiry: 42 };
    assert_eq!(to_vec(&ix).unwrap(), to_vec(&explicit).unwrap());
}

// ---- Memo program v3 ----

#[test]
fn memo_ix_data_is_raw_utf8_no_discriminator() {
    // Memo v3 instruction data = the raw memo UTF-8 bytes. No discriminator, no
    // length prefix (the program reads ix.data directly). Ground-truth from
    // @solana/spl-memo createMemoInstruction (tools/verify-borsh.mjs).
    let bytes = memo_ix_data("sensor reading ok");
    assert_eq!(bytes, b"sensor reading ok", "memo ix data must be raw UTF-8");
    assert_eq!(hex_encode(&bytes), "73656e736f722072656164696e67206f6b");
}

#[test]
fn memo_ix_data_unicode_is_valid_utf8() {
    let bytes = memo_ix_data("DePIN sensor ✓ temp=24.7°C");
    assert_eq!(bytes, "DePIN sensor ✓ temp=24.7°C".as_bytes());
}

// hex helper inline (no extra dep) — encode bytes to lowercase hex.
fn hex_encode(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}