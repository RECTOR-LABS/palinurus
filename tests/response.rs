//! Host tests for the `response` module (slice 7 of palinurus-core).
//!
//! Every shaper must stay within ~200 tokens (the bounty's trap #3: "do not
//! flood the context window"). Token count is approximated as `chars / 4`
//! (≈ 800 chars cap). Judges call `execute` and count tokens — these
//! assertions guard the budget directly.

use palinurus_core::base58::Pubkey;
use palinurus_core::response::{
    estimate_tokens, shape_account_info, shape_attestation_result, shape_signatures,
    short_pubkey, AccountKind,
};
use palinurus_core::rpc::{AccountInfo, TxSummary};

const TOKEN_BUDGET: usize = 200;
const CHAR_CAP: usize = 800; // 200 tokens * 4 chars/token

fn assert_within_budget(s: &str) {
    let chars = s.chars().count();
    let toks = estimate_tokens(s);
    assert!(
        toks <= TOKEN_BUDGET,
        "shaper output {toks} tokens ({chars} chars) exceeds {TOKEN_BUDGET}-token budget:\n{s}"
    );
    assert!(chars <= CHAR_CAP, "shaper output {chars} chars exceeds {CHAR_CAP}-char cap");
}

fn pk(b: u8) -> Pubkey { Pubkey::from_bytes([b; 32]) }

// ---- short_pubkey ----

#[test]
fn short_pubkey_is_4_plus_ellipsis_plus_4() {
    let s = short_pubkey(&Pubkey::SYSTEM);
    // "1111…1111" — 4 + … + 4
    assert!(s.starts_with("1111"));
    assert!(s.ends_with("1111"));
    assert!(s.contains('…'), "should contain an ellipsis: {s}");
    assert!(s.chars().count() <= 13, "short pubkey must be tiny: {s}");
}

// ---- shape_account_info (Nonce) ----

#[test]
fn shape_nonce_account_initialized_is_one_line_within_budget() {
    use solana_program::nonce::state::{DurableNonce, State, Versions};
    let authority = pk(0x20);
    let blockhash = solana_program::hash::Hash::new_from_array([0x11; 32]);
    let durable_nonce = DurableNonce::from_blockhash(&blockhash);
    let versions = Versions::new(State::new_initialized(
        &solana_program::pubkey::Pubkey::new_from_array(authority.to_bytes()),
        durable_nonce,
        5000,
    ));
    let data = bincode::serialize(&versions).unwrap();
    let info = AccountInfo { data, owner: Pubkey::SYSTEM, lamports: 100_000_000, executable: false };
    let s = shape_account_info(&info, AccountKind::Nonce);
    eprintln!("[nonce] {s}");
    assert!(s.contains("nonce"), "should mention nonce: {s}");
    assert!(s.contains("auth"), "should mention authority: {s}");
    assert_within_budget(&s);
    // No raw 80-byte hex dump — the shaped output is far shorter than the raw hex.
    assert!(s.chars().count() < info.data.len() * 2, "must not dump raw bytes as hex");
}

#[test]
fn shape_nonce_account_uninitialized_within_budget() {
    use solana_program::nonce::state::{State, Versions};
    let versions = Versions::new(State::Uninitialized);
    let mut data = bincode::serialize(&versions).unwrap();
    data.resize(80, 0);
    let info = AccountInfo { data, owner: Pubkey::SYSTEM, lamports: 0, executable: false };
    let s = shape_account_info(&info, AccountKind::Nonce);
    eprintln!("[nonce uninit] {s}");
    assert!(s.to_lowercase().contains("uninit"), "should say uninitialized: {s}");
    assert_within_budget(&s);
}

// ---- shape_account_info (Other) ----

#[test]
fn shape_other_account_summary_within_budget() {
    let big = vec![0xABu8; 4096]; // a big blob that would flood context if dumped
    let info = AccountInfo { data: big, owner: pk(0x42), lamports: 9_000_000_000, executable: false };
    let s = shape_account_info(&info, AccountKind::Other);
    eprintln!("[other] {s}");
    assert!(s.contains("4096"), "should mention the data length: {s}");
    assert_within_budget(&s);
    // Must NOT contain the full 4096-byte hex dump.
    assert!(s.chars().count() < 4096, "must not dump the full blob");
}

#[test]
fn shape_attestation_account_summary_within_budget() {
    // An attestation account we don't fully parse yet — show owner + len + prefix.
    let data = vec![0x06u8, 0x05, 0x4a, 0x53, 0x5a, 0x99, 0x29, 0x21, 0x06];
    let info = AccountInfo { data, owner: Pubkey::SAS, lamports: 500_000, executable: false };
    let s = shape_account_info(&info, AccountKind::Attestation);
    eprintln!("[attestation acct] {s}");
    assert!(s.to_lowercase().contains("attest"), "should label attestation: {s}");
    assert_within_budget(&s);
}

// ---- shape_signatures ----

#[test]
fn shape_signatures_small_list_within_budget() {
    let sigs = vec![
        TxSummary { signature: "5TxAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(), err: None, slot: 100, block_time: Some(1753000000) },
        TxSummary { signature: "9TxBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB".to_string(), err: Some("InstructionError".to_string()), slot: 101, block_time: None },
    ];
    let s = shape_signatures(&sigs);
    eprintln!("[sigs small] {s}");
    assert!(s.contains("2"), "should mention count: {s}");
    assert!(s.contains("fail") || s.contains("err") || s.contains("InstructionError"), "should flag the failed tx: {s}");
    assert_within_budget(&s);
}

#[test]
fn shape_signatures_large_list_truncates_within_budget() {
    // 50 signatures — must truncate, not dump all.
    let sigs: Vec<TxSummary> = (0..50).map(|i| TxSummary {
        signature: format!("sig{i:048}"),
        err: None,
        slot: 100 + i,
        block_time: Some(1753000000 + i as i64),
    }).collect();
    let s = shape_signatures(&sigs);
    eprintln!("[sigs large] {s}");
    assert!(s.contains("50"), "should mention total count: {s}");
    assert!(s.contains("more") || s.contains("+"), "should indicate truncation: {s}");
    assert_within_budget(&s);
    // Must not list all 50 signatures.
    assert!(s.chars().count() < 50 * 48, "must not dump all 50 sigs");
}

#[test]
fn shape_signatures_empty_list_within_budget() {
    let s = shape_signatures(&[]);
    eprintln!("[sigs empty] {s}");
    assert_within_budget(&s);
    assert!(s.to_lowercase().contains("no") || s.contains("0"), "should say none: {s}");
}

// ---- shape_attestation_result ----

#[test]
fn shape_attestation_result_within_budget() {
    let sig = "5TxAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    let pda = pk(0x05);
    let nonce = pk(0x20);
    let s = shape_attestation_result(sig, &pda, &nonce);
    eprintln!("[attest result] {s}");
    assert!(s.contains(sig) || s.contains(&short_pubkey(&pda)), "should reference sig or pda: {s}");
    assert!(s.to_lowercase().contains("attest"), "should say attested: {s}");
    assert_within_budget(&s);
}

#[test]
fn estimate_tokens_is_chars_div_4() {
    assert_eq!(estimate_tokens(""), 0);
    assert_eq!(estimate_tokens("abcd"), 1);
    assert_eq!(estimate_tokens("abcdefgh"), 2);
    // ~800 chars → 200 tokens
    let s = "x".repeat(800);
    assert_eq!(estimate_tokens(&s), 200);
}

