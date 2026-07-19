//! Host tests for the `versioned_tx` module (slice 4 of palinurus-core).
//!
//! CONSENSUS-CRITICAL slice. Hand-rolled `VersionedTransaction` V0 construction
//! (unsigned, T1) is verified byte-for-byte against the canonical
//! `solana_program::message::MessageV0::try_compile` + `bincode::serialize(
//! &VersionedMessage::V0(..))` oracle. The oracle drives the real account-key
//! ordering (payer → writable-signers → readonly-signers → writable-non-signers
//! → readonly-non-signers, sorted by pubkey bytes within each category) and the
//! real short-vec (compact-u16) serialization. If our bytes match, our tx is
//! consensus-valid on Solana.

use palinurus_core::base58::Pubkey;
use palinurus_core::versioned_tx::{build_unsigned, serialize, serialize_message, AccountMeta, Instruction};

// ---- Oracle helpers (solana_program) ----

fn sol_pubkey(b: [u8; 32]) -> solana_program::pubkey::Pubkey {
    solana_program::pubkey::Pubkey::new_from_array(b)
}
fn sol_hash(b: [u8; 32]) -> solana_program::hash::Hash {
    solana_program::hash::Hash::new_from_array(b)
}
fn sol_meta(m: &AccountMeta) -> solana_program::instruction::AccountMeta {
    solana_program::instruction::AccountMeta {
        pubkey: sol_pubkey(m.pubkey.to_bytes()),
        is_signer: m.is_signer,
        is_writable: m.is_writable,
    }
}
fn sol_ix(ix: &Instruction) -> solana_program::instruction::Instruction {
    solana_program::instruction::Instruction {
        program_id: sol_pubkey(ix.program_id.to_bytes()),
        accounts: ix.accounts.iter().map(sol_meta).collect(),
        data: ix.data.clone(),
    }
}

/// Build the canonical V0 message bytes (with the 0x80 prefix) via solana_program,
/// then prepend a zero-signatures short-vec byte → the canonical unsigned tx bytes.
fn oracle_unsigned_tx_bytes(ixs: &[Instruction], payer: Pubkey, blockhash: [u8; 32]) -> Vec<u8> {
    let sol_ixs: Vec<_> = ixs.iter().map(sol_ix).collect();
    let v0 = solana_program::message::v0::Message::try_compile(
        &sol_pubkey(payer.to_bytes()),
        &sol_ixs,
        &[],
        sol_hash(blockhash),
    )
    .expect("oracle: try_compile");
    let msg_bytes = bincode::serialize(&solana_program::message::VersionedMessage::V0(v0))
        .expect("oracle: bincode serialize VersionedMessage::V0");
    // Full tx = short-vec(0 signatures) + message bytes. short-vec(0) = [0x00].
    let mut out = vec![0x00];
    out.extend_from_slice(&msg_bytes);
    out
}

// ---- Fixed test pubkeys (bytes chosen so naive insertion-order would differ
// from canonical sorted order — the oracle catches a wrong ordering) ----

fn pk(b: u8) -> Pubkey { Pubkey::from_bytes([b; 32]) }
fn blockhash() -> [u8; 32] { [0x11; 32] }

// A realistic SAS create_attestation-shaped instruction: 6 accounts, mixed roles.
fn sas_attest_ix(payer: Pubkey, authority: Pubkey, credential: Pubkey, schema: Pubkey, attestation: Pubkey, system: Pubkey, data: Vec<u8>) -> Instruction {
    Instruction {
        program_id: Pubkey::SAS,
        accounts: vec![
            AccountMeta { pubkey: payer, is_signer: true, is_writable: true },       // payer WS
            AccountMeta { pubkey: authority, is_signer: true, is_writable: false },   // authority RS
            AccountMeta { pubkey: credential, is_signer: false, is_writable: false }, // credential R
            AccountMeta { pubkey: schema, is_signer: false, is_writable: false },     // schema R
            AccountMeta { pubkey: attestation, is_signer: false, is_writable: true }, // attestation W
            AccountMeta { pubkey: system, is_signer: false, is_writable: false },     // system R
        ],
        data,
    }
}

#[test]
fn unsigned_tx_matches_oracle_single_sas_ix() {
    let payer = pk(0x01);
    let authority = pk(0x02);
    let credential = pk(0x03);
    let schema = pk(0x04);
    let attestation = pk(0x05);
    let system = Pubkey::SYSTEM;
    let data = vec![6u8, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE]; // arbitrary ix data
    let ix = sas_attest_ix(payer, authority, credential, schema, attestation, system, data);
    let ixs = vec![ix];

    let my = serialize(&build_unsigned(&ixs, payer, blockhash()));
    let refb = oracle_unsigned_tx_bytes(&ixs, payer, blockhash());
    assert_eq!(my, refb, "unsigned tx bytes must match solana_program oracle byte-for-byte (single SAS ix)");
}

#[test]
fn unsigned_tx_matches_oracle_two_ixs_with_shared_accounts() {
    let payer = pk(0x01);
    let a = pk(0x10);
    let b = pk(0x20);
    let prog1 = pk(0x30);
    let prog2 = pk(0x40);
    let ixs = vec![
        Instruction { program_id: prog1, accounts: vec![
            AccountMeta { pubkey: payer, is_signer: true, is_writable: true },
            AccountMeta { pubkey: a, is_signer: false, is_writable: true },
            AccountMeta { pubkey: b, is_signer: false, is_writable: false },
        ], data: vec![1, 2, 3] },
        Instruction { program_id: prog2, accounts: vec![
            AccountMeta { pubkey: a, is_signer: false, is_writable: true }, // shared, dedup
            AccountMeta { pubkey: b, is_signer: false, is_writable: false }, // shared, dedup
            AccountMeta { pubkey: payer, is_signer: true, is_writable: true }, // shared with payer
        ], data: vec![9] },
    ];

    let my = serialize(&build_unsigned(&ixs, payer, blockhash()));
    let refb = oracle_unsigned_tx_bytes(&ixs, payer, blockhash());
    assert_eq!(my, refb, "unsigned tx bytes must match oracle (two ixs, shared accounts, dedup + ordering)");
}

#[test]
fn account_key_ordering_is_payer_then_signers_then_writable_then_readonly() {
    // Construct an ix whose account-metas are in INTENTIONALLY non-canonical
    // order, with pubkeys chosen so canonical (sorted) order differs from
    // insertion order. A naive impl that keeps insertion order would fail.
    // pubkey bytes: payer=0x01, readonly_signer=0x07, writable_non_signer=0x03, readonly_non_signer=0x09
    // canonical order: writable_signers [payer 0x01] | readonly_signers [0x07] | writable_non_signers [0x03] | readonly_non_signers [0x09 + program 0xff...]
    let payer = pk(0x01);
    let readonly_signer = pk(0x07);
    let writable_non_signer = pk(0x03);
    let readonly_non_signer = pk(0x09);
    let program = pk(0xFF);
    // Insert in scrambled order:
    let ixs = vec![Instruction {
        program_id: program,
        accounts: vec![
            AccountMeta { pubkey: readonly_non_signer, is_signer: false, is_writable: false },
            AccountMeta { pubkey: writable_non_signer, is_signer: false, is_writable: true },
            AccountMeta { pubkey: readonly_signer, is_signer: true, is_writable: false },
            AccountMeta { pubkey: payer, is_signer: true, is_writable: true },
        ],
        data: vec![],
    }];

    let my = serialize(&build_unsigned(&ixs, payer, blockhash()));
    let refb = oracle_unsigned_tx_bytes(&ixs, payer, blockhash());
    assert_eq!(my, refb, "account-key ordering must match canonical 4-category sort, not insertion order");
}

#[test]
fn empty_accounts_ix_matches_oracle() {
    // Memo-style ix: program + data, no accounts.
    let payer = pk(0x01);
    let ixs = vec![Instruction {
        program_id: Pubkey::MEMO,
        accounts: vec![],
        data: b"sensor ok".to_vec(),
    }];
    let my = serialize(&build_unsigned(&ixs, payer, blockhash()));
    let refb = oracle_unsigned_tx_bytes(&ixs, payer, blockhash());
    assert_eq!(my, refb, "empty-accounts ix must match oracle");
}

#[test]
fn unsigned_tx_has_zero_signatures_prefix() {
    let payer = pk(0x01);
    let ixs = vec![Instruction { program_id: Pubkey::MEMO, accounts: vec![], data: vec![] }];
    let my = serialize(&build_unsigned(&ixs, payer, blockhash()));
    assert_eq!(my[0], 0x00, "unsigned tx: short-vec(0) signatures prefix = 0x00");
    assert_eq!(my[1], 0x80, "V0 message prefix byte = 0x80");
}

#[test]
fn v0_message_header_counts_match_oracle() {
    // 2 signers (payer WS + authority RS), 1 readonly signer, 1 readonly non-signer (system).
    let payer = pk(0x01);
    let authority = pk(0x02);
    let credential = pk(0x03);
    let schema = pk(0x04);
    let attestation = pk(0x05);
    let ix = sas_attest_ix(payer, authority, credential, schema, attestation, Pubkey::SYSTEM, vec![6]);
    let tx = build_unsigned(&[ix], payer, blockhash());
    // header: num_required_signatures=2, num_readonly_signed=1, num_readonly_unsigned=2 (credential+schema+system = 3? check oracle)
    // We assert our header equals the oracle's by full-byte equality above; here sanity-check the counts.
    let h = &tx.message.header;
    assert_eq!(h.num_required_signatures, 2, "payer + authority");
    assert_eq!(h.num_readonly_signed_accounts, 1, "authority is readonly signer");
    // readonly non-signers: credential, schema, system, SAS(program) → 4
    assert_eq!(h.num_readonly_unsigned_accounts, 4, "credential + schema + system + SAS program");
}

#[test]
fn serialize_message_is_tx_bytes_minus_zero_sig_prefix() {
    // For an unsigned tx, serialize(tx) = [0x00 short-vec] + serialize_message(msg).
    // serialize_message yields the bytes a signer hashes (0x80 prefix + V0 body).
    let payer = pk(0x01);
    let ixs = vec![sas_attest_ix(payer, pk(0x02), pk(0x03), pk(0x04), pk(0x05), Pubkey::SYSTEM, vec![6])];
    let tx = build_unsigned(&ixs, payer, blockhash());
    let tx_bytes = serialize(&tx);
    let msg_bytes = serialize_message(&tx.message);
    assert_eq!(tx_bytes[0], 0x00, "zero sigs prefix");
    assert_eq!(&tx_bytes[1..], &msg_bytes[..], "tx bytes (after sig prefix) == serialize_message");
    assert_eq!(msg_bytes[0], 0x80, "message starts with V0 prefix");
}