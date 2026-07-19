//! Versioned transaction (V0) construction — unsigned, T1 custody.
//!
//! Hand-rolled, `wasm32-wasip2`-friendly reimplementation of the pieces of
//! `solana-sdk`/`solana-message` that a Solana tool plugin needs to **build an
//! unsigned versioned transaction** for a human / Squads multisig to sign.
//! `solana-sdk` cannot compile inside a WIT component, so this module
//! hand-rolls the minimal V0 message + transaction serialization.
//!
//! ## Consensus-critical — oracle-verified
//!
//! The account-key ordering and the short-vec (compact-u16) serialization are
//! consensus-critical: a wrong order or a wrong length prefix produces a
//! valid-looking but consensus-invalid transaction. Every byte is verified
//! against the canonical `solana_program::message::v0::Message::try_compile`
//! and `bincode::serialize(&VersionedMessage::V0(..))` oracle in
//! `tests/versioned_tx.rs` — the same byte-for-byte rigor as the PDA spike.
//!
//! ## Account-key ordering (matches `solana_message::compiled_keys`)
//!
//! 1. **writable signers** — payer first, then other signer+writable keys
//!    (sorted by pubkey bytes ascending, BTreeMap order).
//! 2. **readonly signers** — signer && !writable (sorted).
//! 3. **writable non-signers** — !signer && writable (sorted).
//! 4. **readonly non-signers** — !signer && !writable (sorted; includes program ids).
//!
//! Header: `num_required_signatures` = signer count, `num_readonly_signed_accounts`
//! = readonly-signer count, `num_readonly_unsigned_accounts` = readonly-non-signer count.
//!
//! ## Wire format (V0)
//!
//! `VersionedTransaction` = `[short-vec sig count][sig0..sigN][0x80][V0 message body]`.
//! V0 message body = `[header 3B][short-vec account_keys][32B blockhash]
//! [short-vec instructions][short-vec address_table_lookups]`.
//! `CompiledInstruction` = `[u8 program_id_index][short-vec accounts][short-vec data]`.

use crate::base58::Pubkey;
use std::collections::BTreeMap;

/// A Solana blockhash (32 bytes).
pub type Blockhash = [u8; 32];

/// An account used by an instruction, with its signer / writable flags.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AccountMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl AccountMeta {
    pub fn signer_writable(pubkey: Pubkey) -> Self { Self { pubkey, is_signer: true, is_writable: true } }
    pub fn signer_readonly(pubkey: Pubkey) -> Self { Self { pubkey, is_signer: true, is_writable: false } }
    pub fn writable(pubkey: Pubkey) -> Self { Self { pubkey, is_signer: false, is_writable: true } }
    pub fn readonly(pubkey: Pubkey) -> Self { Self { pubkey, is_signer: false, is_writable: false } }
}

/// A Solana instruction: program + accounts + data.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Instruction {
    pub program_id: Pubkey,
    pub accounts: Vec<AccountMeta>,
    pub data: Vec<u8>,
}

/// Message header — describes the account-key layout (3 bytes).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct MessageHeader {
    pub num_required_signatures: u8,
    pub num_readonly_signed_accounts: u8,
    pub num_readonly_unsigned_accounts: u8,
}

/// A compact instruction (account indices into the message's `account_keys`).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CompiledInstruction {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
    pub data: Vec<u8>,
}

/// An address-table lookup (V0). Unused by our plugins (no ALTs) but kept for
/// format completeness.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MessageAddressTableLookup {
    pub account_key: Pubkey,
    pub writable_indexes: Vec<u8>,
    pub readonly_indexes: Vec<u8>,
}

/// A V0 message.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MessageV0 {
    pub header: MessageHeader,
    pub account_keys: Vec<Pubkey>,
    pub recent_blockhash: Blockhash,
    pub instructions: Vec<CompiledInstruction>,
    pub address_table_lookups: Vec<MessageAddressTableLookup>,
}

/// A versioned transaction. For T1 (unsigned) the `signatures` vec is empty;
/// a human / Squads multisig signs the message hash and submits.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VersionedTransaction {
    pub signatures: Vec<[u8; 64]>,
    pub message: MessageV0,
}

/// Per-key signer/writable flags accumulated across all instructions + the payer.
#[derive(Default, Clone, Copy)]
struct KeyMeta {
    is_signer: bool,
    is_writable: bool,
}

/// Build an **unsigned** V0 versioned transaction (T1) from `instructions`,
/// a `payer`, and a `blockhash`. Signatures are empty — a human or Squads
/// multisig signs the message hash and submits.
///
/// Account keys are ordered canonical-Solana (payer → writable-signers →
/// readonly-signers → writable-non-signers → readonly-non-signers, sorted by
/// pubkey bytes within each category). Throws via panic on >255 accounts
/// (mirrors `solana_message`'s `CompileError::AccountIndexOverflow`); unreachable
/// for our plugins (≤ ~10 accounts).
pub fn build_unsigned(ixs: &[Instruction], payer: Pubkey, blockhash: Blockhash) -> VersionedTransaction {
    let mut key_map: BTreeMap<Pubkey, KeyMeta> = BTreeMap::new();

    // Accumulate signer/writable flags across all instructions. Program ids are
    // inserted (so they get an index) but default to !signer && !writable →
    // readonly-non-signer, matching solana_message (is_invoked doesn't affect ordering).
    for ix in ixs {
        key_map.entry(ix.program_id).or_default();
        for am in &ix.accounts {
            let m = key_map.entry(am.pubkey).or_default();
            m.is_signer |= am.is_signer;
            m.is_writable |= am.is_writable;
        }
    }

    // Payer is always a writable signer.
    {
        let m = key_map.entry(payer).or_default();
        m.is_signer = true;
        m.is_writable = true;
    }
    // Remove payer from the map so the category filters don't re-include it;
    // payer is prepended to writable_signers explicitly.
    key_map.remove(&payer);

    let writable_signers: Vec<Pubkey> = std::iter::once(payer)
        .chain(
            key_map
                .iter()
                .filter(|(_, m)| m.is_signer && m.is_writable)
                .map(|(k, _)| *k),
        )
        .collect();
    let readonly_signers: Vec<Pubkey> = key_map
        .iter()
        .filter(|(_, m)| m.is_signer && !m.is_writable)
        .map(|(k, _)| *k)
        .collect();
    let writable_non_signers: Vec<Pubkey> = key_map
        .iter()
        .filter(|(_, m)| !m.is_signer && m.is_writable)
        .map(|(k, _)| *k)
        .collect();
    let readonly_non_signers: Vec<Pubkey> = key_map
        .iter()
        .filter(|(_, m)| !m.is_signer && !m.is_writable)
        .map(|(k, _)| *k)
        .collect();

    let account_keys: Vec<Pubkey> = writable_signers
        .iter()
        .chain(readonly_signers.iter())
        .chain(writable_non_signers.iter())
        .chain(readonly_non_signers.iter())
        .copied()
        .collect();

    let header = MessageHeader {
        num_required_signatures: u8::try_from(writable_signers.len() + readonly_signers.len())
            .expect("<=255 signers"),
        num_readonly_signed_accounts: u8::try_from(readonly_signers.len()).expect("<=255 readonly signers"),
        num_readonly_unsigned_accounts: u8::try_from(readonly_non_signers.len())
            .expect("<=255 readonly non-signers"),
    };

    // Index map for compiling instructions.
    let mut index_map: BTreeMap<Pubkey, u8> = BTreeMap::new();
    for (i, k) in account_keys.iter().enumerate() {
        index_map.insert(*k, u8::try_from(i).expect("<=255 account keys"));
    }

    let instructions: Vec<CompiledInstruction> = ixs
        .iter()
        .map(|ix| {
            let accounts: Vec<u8> = ix
                .accounts
                .iter()
                .map(|am| *index_map.get(&am.pubkey).expect("account key present in account_keys"))
                .collect();
            CompiledInstruction {
                program_id_index: *index_map.get(&ix.program_id).expect("program id present in account_keys"),
                accounts,
                data: ix.data.clone(),
            }
        })
        .collect();

    let message = MessageV0 {
        header,
        account_keys,
        recent_blockhash: blockhash,
        instructions,
        address_table_lookups: Vec::new(),
    };
    VersionedTransaction { signatures: Vec::new(), message }
}

/// Serialize a `VersionedTransaction` to the on-wire bytes (bincode-compatible,
/// matches `solana_sdk::versioned_transaction::VersionedTransaction::serialize`).
///
/// Layout: `[short-vec sig count][sig0..sigN][0x80][V0 message body]`.
pub fn serialize(tx: &VersionedTransaction) -> Vec<u8> {
    let mut out = Vec::new();
    encode_short_vec(tx.signatures.len(), &mut out);
    for sig in &tx.signatures {
        out.extend_from_slice(sig);
    }
    encode_v0_message(&tx.message, &mut out);
    out
}

/// Serialize a V0 message with the `0x80` version prefix (bincode-compatible,
/// matches `bincode::serialize(&VersionedMessage::V0(msg))`).
pub fn serialize_message(msg: &MessageV0) -> Vec<u8> {
    let mut out = Vec::new();
    encode_v0_message(msg, &mut out);
    out
}

fn encode_v0_message(msg: &MessageV0, out: &mut Vec<u8>) {
    out.push(0x80); // V0 version prefix
    // header (3 bytes, bincode struct = fields in order, no wrapper)
    out.push(msg.header.num_required_signatures);
    out.push(msg.header.num_readonly_signed_accounts);
    out.push(msg.header.num_readonly_unsigned_accounts);
    // account_keys (short-vec + 32B each)
    encode_short_vec(msg.account_keys.len(), out);
    for k in &msg.account_keys {
        out.extend_from_slice(k.as_bytes());
    }
    // recent_blockhash (32 raw bytes)
    out.extend_from_slice(&msg.recent_blockhash);
    // instructions (short-vec + each compiled instruction)
    encode_short_vec(msg.instructions.len(), out);
    for ci in &msg.instructions {
        out.push(ci.program_id_index);
        encode_short_vec(ci.accounts.len(), out);
        out.extend_from_slice(&ci.accounts);
        encode_short_vec(ci.data.len(), out);
        out.extend_from_slice(&ci.data);
    }
    // address_table_lookups (short-vec + each)
    encode_short_vec(msg.address_table_lookups.len(), out);
    for alt in &msg.address_table_lookups {
        out.extend_from_slice(alt.account_key.as_bytes());
        encode_short_vec(alt.writable_indexes.len(), out);
        out.extend_from_slice(&alt.writable_indexes);
        encode_short_vec(alt.readonly_indexes.len(), out);
        out.extend_from_slice(&alt.readonly_indexes);
    }
}

/// Encode a length as a Solana short-vec (compact-u16) prefix:
/// `< 0x80` → 1 byte; `< 0x4000` → 2 bytes; else 3 bytes. Matches
/// `solana_short_vec::ShortU16` bincode serde exactly.
fn encode_short_vec(len: usize, out: &mut Vec<u8>) {
    let len = u16::try_from(len).expect("short-vec length fits in u16 (<=65535)");
    if len < 0x80 {
        out.push(len as u8);
    } else if len < 0x4000 {
        out.push(0x80 | (len & 0x7f) as u8);
        out.push((len >> 7) as u8);
    } else {
        out.push(0x80 | (len & 0x7f) as u8);
        out.push(0x80 | ((len >> 7) & 0x7f) as u8);
        out.push((len >> 14) as u8);
    }
}