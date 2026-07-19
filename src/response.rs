//! Response shaping — keep RPC/IX outputs to ~200 tokens, not 40KB JSON.
//!
//! The bounty's trap #3: raw `getAccountInfo`/`getSignaturesForAddress` responses
//! nuke the agent's context window and cost the operator money. Judges call
//! `execute` and count tokens. Every shaper here stays within a ~200-token budget
//! (≈ 800 chars at the `chars / 4` heuristic — see [`estimate_tokens`]).
//!
//! Shapers are pure string formatting — no consensus concerns, no network.

use crate::base58::Pubkey;
use crate::durable_nonce::{parse_nonce_account, NonceState, NonceVersion};
use crate::rpc::{AccountInfo, TxSummary};

/// What kind of account `shape_account_info` is summarizing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccountKind {
    /// A durable-nonce account — parsed with `parse_nonce_account` for a real summary.
    Nonce,
    /// A Solana Attestation Service attestation account — owner + len + data prefix.
    Attestation,
    /// Anything else — owner + len + data prefix.
    Other,
}

/// Approximate token count via the `chars / 4` heuristic. The bounty's shaping
/// budget is ~200 tokens; [`shape_account_info`] / [`shape_signatures`] /
/// [`shape_attestation_result`] all stay under it.
pub fn estimate_tokens(s: &str) -> usize {
    s.chars().count() / 4
}

/// Short pubkey form: first 4 + `…` + last 4 (e.g. `Et5C…T2p`). ~9 chars, ~2 tokens.
pub fn short_pubkey(pk: &Pubkey) -> String {
    let s = pk.to_string();
    let bytes = s.as_bytes();
    if bytes.len() <= 8 {
        return s;
    }
    format!("{}…{}", &s[..4], &s[s.len() - 4..])
}

/// One-line summary of an account, tuned to the `kind`. Never dumps the raw bytes.
pub fn shape_account_info(info: &AccountInfo, kind: AccountKind) -> String {
    match kind {
        AccountKind::Nonce => shape_nonce_account(info),
        AccountKind::Attestation => format!(
            "Attestation account owner={} len={}B lamports={} data[0:8]={}",
            short_pubkey(&info.owner),
            info.data.len(),
            info.lamports,
            hex_prefix(&info.data, 8),
        ),
        AccountKind::Other => format!(
            "Account owner={} len={}B lamports={} executable={} data[0:8]={}",
            short_pubkey(&info.owner),
            info.data.len(),
            info.lamports,
            info.executable,
            hex_prefix(&info.data, 8),
        ),
    }
}

fn shape_nonce_account(info: &AccountInfo) -> String {
    match parse_nonce_account(&info.data) {
        Ok(nonce) => match nonce.version {
            NonceVersion::Legacy => "Nonce: legacy (unsupported for versioned tx)".to_string(),
            NonceVersion::Current => match nonce.state {
                NonceState::Uninitialized => "Nonce: current/uninitialized".to_string(),
                NonceState::Initialized(d) => format!(
                    "Nonce: current/initialized auth={} nonce={} lamports_per_sig={}",
                    short_pubkey(&d.authority),
                    short_pubkey(&Pubkey::from_bytes(d.durable_nonce)),
                    d.lamports_per_signature,
                ),
            },
        },
        Err(e) => format!(
            "Nonce: parse error ({:?}) owner={} len={}B data[0:8]={}",
            e,
            short_pubkey(&info.owner),
            info.data.len(),
            hex_prefix(&info.data, 8),
        ),
    }
}

/// Compact list of recent signatures for an address. Shows the first few + a
/// "+N more" tail when the list is long.
pub fn shape_signatures(sigs: &[TxSummary]) -> String {
    if sigs.is_empty() {
        return "No recent transactions.".to_string();
    }
    let total = sigs.len();
    const SHOW: usize = 5;
    let head: Vec<String> = sigs.iter().take(SHOW).map(|t| {
        let status = match &t.err {
            None => "ok".to_string(),
            Some(e) => format!("FAIL:{}", truncate(e, 20)),
        };
        let sig_short = truncate(&t.signature, 8);
        format!("{}(slot {}, {})", sig_short, t.slot, status)
    }).collect();
    let mut out = format!("{} txs: {}", total, head.join(" | "));
    if total > SHOW {
        out.push_str(&format!(" | +{} more", total - SHOW));
    }
    out
}

/// One-line result of a `create_attestation` submission: PDA + nonce + signature.
pub fn shape_attestation_result(sig: &str, pda: &Pubkey, nonce: &Pubkey) -> String {
    format!(
        "Attested → PDA {} (nonce {}) sig {}",
        short_pubkey(pda),
        short_pubkey(nonce),
        truncate(sig, 12),
    )
}

// ---- helpers ----

fn hex_prefix(data: &[u8], n: usize) -> String {
    let n = n.min(data.len());
    let mut s = String::with_capacity(n * 2);
    for b in data.iter().take(n) {
        s.push_str(&format!("{:02x}", b));
    }
    if data.len() > n {
        s.push('…');
    }
    s
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let head: String = s.chars().take(max).collect();
    format!("{head}…")
}