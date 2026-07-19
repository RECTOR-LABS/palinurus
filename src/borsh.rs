//! Borsh instruction-data helpers for Solana instructions the Palinurus plugins emit.
//!
//! Pure-Rust, `wasm32-wasip2`-friendly. The `borsh` crate handles primitive
//! (u8 / u32 / u64 / i64 / Vec / arrays) serialization; this module defines the
//! Solana-specific structs + helpers, verified byte-for-byte against canonical
//! reference encoders:
//!
//! - [`CreateAttestationIxData`] — the Solana Attestation Service (SAS)
//!   `create_attestation` instruction DATA. Layout verified 2026-07-19 against
//!   the sas-lib Codama-generated codec (`tools/verify-borsh.mjs` oracle,
//!   `sas-lib@1.0.10`):
//!   `[u8 discriminator=6] [32-byte nonce] [u32 LE len][N bytes data] [i64 LE expiry]`
//! - [`memo_ix_data`] — the SPL Memo program v3 instruction DATA = raw UTF-8
//!   bytes (no discriminator, no length prefix; ground-truth from
//!   `@solana/spl-memo` `createMemoInstruction`).
//!
//! The nonce is a `Pubkey` (used as an attestation-PDA seed for uniqueness — the
//! replay guard), NOT a u64 counter. `expiry` is a signed `i64` unix timestamp.
//! `data` is the schema-encoded attestation payload (`serializeAttestationData`
//! in sas-lib) — produced by the plugin, opaque to this struct.

use crate::base58::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};

/// Solana Attestation Service `create_attestation` instruction DATA.
///
/// Borsh layout (matches sas-lib `getCreateAttestationInstructionDataEncoder`):
/// `u8 discriminator` (=6) · `Pubkey nonce` (32 B) · `Vec<u8> data` (u32 LE len + bytes) · `i64 expiry` (LE).
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug)]
pub struct CreateAttestationIxData {
    /// Instruction discriminator — `CREATE_ATTESTATION_DISCRIMINATOR = 6`.
    pub discriminator: u8,
    /// Nonce — a `Pubkey` used as an attestation-PDA seed (uniqueness / replay guard).
    pub nonce: Pubkey,
    /// Schema-encoded attestation payload (opaque bytes; the plugin produces them
    /// via the Schema's Borsh layout).
    pub data: Vec<u8>,
    /// Expiry — signed `i64` unix timestamp (seconds).
    pub expiry: i64,
}

impl CreateAttestationIxData {
    /// The SAS `create_attestation` instruction discriminator (u8 = 6).
    pub const DISCRIMINATOR: u8 = 6;

    /// Convenience constructor (discriminator is filled in automatically).
    pub fn new(nonce: Pubkey, data: Vec<u8>, expiry: i64) -> Self {
        Self { discriminator: Self::DISCRIMINATOR, nonce, data, expiry }
    }

    /// Serialize to the on-wire instruction DATA bytes.
    pub fn to_ix_bytes(&self) -> Vec<u8> {
        borsh::to_vec(self).expect("CreateAttestationIxData: borsh serialize is infallible for these types")
    }
}

/// SPL Memo program v3 instruction DATA — the raw memo UTF-8 bytes.
///
/// The memo program reads `ix.data` directly: no discriminator, no length
/// prefix. Returns the memo string as a byte vector.
pub fn memo_ix_data(memo: &str) -> Vec<u8> {
    memo.as_bytes().to_vec()
}