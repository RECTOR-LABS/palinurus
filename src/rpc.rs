//! Solana JSON-RPC 2.0 client over `waki` (blocking `wasi:http`).
//!
//! The host grants the `http_client` capability; TLS is handled host-side. Only
//! the methods our plugins call: `getLatestBlockhash`, `getAccountInfo`,
//! `getSignaturesForAddress`, `sendTransaction`.
//!
//! ## Testable seam
//!
//! The JSON-RPC request-building ([`rpc_request`]) and response-parsing
//! ([`rpc_result`], [`parse_blockhash`], [`parse_account_info`],
//! [`parse_signatures`], [`parse_send_tx`]) logic is **pure** and fully
//! host-testable via [`MockRpc`] (scripted raw JSON-RPC responses → typed Rust).
//! [`WakiRpc`] shares the same parse layer and only differs in transport
//! (waki POST) — it is exercised live inside a wasm32-wasip2 component with a
//! wasi:http runtime, never on host. No live network in `cargo test`.
//!
//! ## API-key auth
//!
//! `waki` supports custom headers (`RequestBuilder::header`), so an RPC API key
//! is sent as `Authorization: Bearer <key>` when `api_key` is `Some`. Providers
//! that use URL-path auth (Helius, QuickNode) just embed the key in the
//! `endpoint` URL and leave `api_key` = `None`. Endpoint + key come from plugin
//! config (`config_read`), never hardcoded.

use crate::base58::Pubkey;
use crate::versioned_tx::Blockhash;
use base64::prelude::{BASE64_STANDARD, Engine as _};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

/// `getLatestBlockhash` result.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BlockhashInfo {
    pub blockhash: Blockhash,
    pub last_valid_block_height: u64,
}

/// `getAccountInfo` result (the account exists). `data` is the raw decoded
/// account bytes (base64-decoded from the RPC `[<b64>, "base64"]` form).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AccountInfo {
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub lamports: u64,
    pub executable: bool,
}

/// One entry from `getSignaturesForAddress`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TxSummary {
    pub signature: String,
    /// `null` on success; a string/object describing the failure on a failed tx.
    pub err: Option<String>,
    pub slot: u64,
    pub block_time: Option<i64>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RpcError {
    /// waki / wasi:http transport failure.
    Transport(String),
    /// serde_json (de)serialization failure.
    Json(String),
    /// A JSON-RPC `error` object returned by the RPC server.
    Rpc { code: i64, message: String },
    /// Response envelope missing `result`/`error`, or a required field absent.
    Unexpected(String),
    /// A base58 string in the response didn't decode to a 32-byte pubkey/blockhash.
    Base58(String),
    /// A base64 account-data blob didn't decode.
    Base64(String),
}

/// The Solana JSON-RPC trait. Blocking. Mockable via [`MockRpc`].
pub trait Rpc {
    fn get_latest_blockhash(&self) -> Result<BlockhashInfo, RpcError>;
    fn get_account_info(&self, pubkey: &Pubkey) -> Result<Option<AccountInfo>, RpcError>;
    fn get_signatures_for_address(&self, pubkey: &Pubkey, limit: usize) -> Result<Vec<TxSummary>, RpcError>;
    /// Submit a signed transaction (raw wire bytes). Returns the transaction signature.
    /// T1 plugins don't call this (they return unsigned tx bytes); T2 does.
    fn send_transaction(&self, tx: &[u8]) -> Result<String, RpcError>;
}

// ---- Pure JSON-RPC builders / parsers (host-testable) ----

/// Build a JSON-RPC 2.0 request envelope.
pub fn rpc_request(id: u64, method: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
}

/// Extract the `result` from a JSON-RPC response, or surface the `error` object.
pub fn rpc_result(v: &Value) -> Result<&Value, RpcError> {
    if let Some(err) = v.get("error") {
        return Err(RpcError::Rpc {
            code: err.get("code").and_then(|c| c.as_i64()).unwrap_or(0),
            message: err.get("message").and_then(|m| m.as_str()).unwrap_or("").to_string(),
        });
    }
    v.get("result")
        .ok_or_else(|| RpcError::Unexpected("missing `result` or `error` in JSON-RPC response".to_string()))
}

/// Parse a `getLatestBlockhash` `result` value.
pub fn parse_blockhash(v: &Value) -> Result<BlockhashInfo, RpcError> {
    let value = v
        .get("value")
        .ok_or_else(|| RpcError::Unexpected("missing result.value".to_string()))?;
    let bh_str = value
        .get("blockhash")
        .and_then(|b| b.as_str())
        .ok_or_else(|| RpcError::Unexpected("missing blockhash".to_string()))?;
    let blockhash = Pubkey::from_str(bh_str)
        .map_err(|e| RpcError::Base58(e.to_string()))?
        .to_bytes();
    let last_valid_block_height = value
        .get("lastValidBlockHeight")
        .and_then(|h| h.as_u64())
        .ok_or_else(|| RpcError::Unexpected("missing lastValidBlockHeight".to_string()))?;
    Ok(BlockhashInfo { blockhash, last_valid_block_height })
}

/// Parse a `getAccountInfo` `result` value. `value: null` → `Ok(None)`.
pub fn parse_account_info(v: &Value) -> Result<Option<AccountInfo>, RpcError> {
    let value = match v.get("value") {
        None => return Err(RpcError::Unexpected("missing result.value".to_string())),
        Some(Value::Null) => return Ok(None),
        Some(v) => v,
    };
    let data_arr = value
        .get("data")
        .ok_or_else(|| RpcError::Unexpected("missing data".to_string()))?;
    let b64 = data_arr
        .get(0)
        .and_then(|d| d.as_str())
        .ok_or_else(|| RpcError::Unexpected("data[0] not a string".to_string()))?;
    let data = BASE64_STANDARD
        .decode(b64)
        .map_err(|e| RpcError::Base64(e.to_string()))?;
    let owner_str = value
        .get("owner")
        .and_then(|o| o.as_str())
        .ok_or_else(|| RpcError::Unexpected("missing owner".to_string()))?;
    let owner = Pubkey::from_str(owner_str).map_err(|e| RpcError::Base58(e.to_string()))?;
    let lamports = value
        .get("lamports")
        .and_then(|l| l.as_u64())
        .ok_or_else(|| RpcError::Unexpected("missing lamports".to_string()))?;
    let executable = value.get("executable").and_then(|e| e.as_bool()).unwrap_or(false);
    Ok(Some(AccountInfo { data, owner, lamports, executable }))
}

/// Parse a `getSignaturesForAddress` `result` array.
pub fn parse_signatures(v: &Value) -> Result<Vec<TxSummary>, RpcError> {
    let arr = v
        .as_array()
        .ok_or_else(|| RpcError::Unexpected("signatures result not an array".to_string()))?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let signature = item
            .get("signature")
            .and_then(|s| s.as_str())
            .ok_or_else(|| RpcError::Unexpected("missing signature".to_string()))?
            .to_string();
        let err = match item.get("err") {
            None | Some(Value::Null) => None,
            Some(e) => Some(if let Some(s) = e.as_str() { s.to_string() } else { e.to_string() }),
        };
        let slot = item
            .get("slot")
            .and_then(|s| s.as_u64())
            .ok_or_else(|| RpcError::Unexpected("missing slot".to_string()))?;
        let block_time = item.get("blockTime").and_then(|b| b.as_i64());
        out.push(TxSummary { signature, err, slot, block_time });
    }
    Ok(out)
}

/// Parse a `sendTransaction` `result` (the signature string).
pub fn parse_send_tx(v: &Value) -> Result<String, RpcError> {
    v.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| RpcError::Unexpected("sendTransaction result not a string".to_string()))
}

// ---- WakiRpc: the waki transport ----

/// A Solana JSON-RPC client backed by `waki` (blocking `wasi:http`).
pub struct WakiRpc {
    endpoint: String,
    api_key: Option<String>,
    id: AtomicU64,
}

impl WakiRpc {
    /// Construct with an RPC endpoint URL and an optional API key. The key, if
    /// present, is sent as `Authorization: Bearer <key>`. For providers that
    /// embed the key in the URL path (Helius/QuickNode), leave `api_key` = None.
    pub fn new(endpoint: String, api_key: Option<String>) -> Self {
        Self { endpoint, api_key, id: AtomicU64::new(0) }
    }
    pub fn endpoint(&self) -> &str { &self.endpoint }
    pub fn api_key(&self) -> Option<&str> { self.api_key.as_deref() }

    fn next_id(&self) -> u64 { self.id.fetch_add(1, Ordering::Relaxed) + 1 }

    fn post_json(&self, req: &Value) -> Result<Value, RpcError> {
        // `.json()` sets Content-Type: application/json + serializes the body.
        let mut builder = waki::Client::new().post(&self.endpoint);
        if let Some(key) = &self.api_key {
            let auth = format!("Bearer {key}");
            builder = builder.header("Authorization", auth.as_str());
        }
        let resp = builder
            .json(req)
            .send()
            .map_err(|e| RpcError::Transport(e.to_string()))?;
        resp.json::<Value>().map_err(|e| RpcError::Json(e.to_string()))
    }
}

impl Rpc for WakiRpc {
    fn get_latest_blockhash(&self) -> Result<BlockhashInfo, RpcError> {
        let req = rpc_request(self.next_id(), "getLatestBlockhash", json!([]));
        let resp = self.post_json(&req)?;
        parse_blockhash(rpc_result(&resp)?)
    }
    fn get_account_info(&self, pubkey: &Pubkey) -> Result<Option<AccountInfo>, RpcError> {
        let req = rpc_request(
            self.next_id(),
            "getAccountInfo",
            json!([pubkey.to_string(), { "encoding": "base64" }]),
        );
        let resp = self.post_json(&req)?;
        parse_account_info(rpc_result(&resp)?)
    }
    fn get_signatures_for_address(&self, pubkey: &Pubkey, limit: usize) -> Result<Vec<TxSummary>, RpcError> {
        let req = rpc_request(
            self.next_id(),
            "getSignaturesForAddress",
            json!([pubkey.to_string(), { "limit": limit }]),
        );
        let resp = self.post_json(&req)?;
        parse_signatures(rpc_result(&resp)?)
    }
    fn send_transaction(&self, tx: &[u8]) -> Result<String, RpcError> {
        let b64 = BASE64_STANDARD.encode(tx);
        let req = rpc_request(self.next_id(), "sendTransaction", json!([b64, { "encoding": "base64" }]));
        let resp = self.post_json(&req)?;
        parse_send_tx(rpc_result(&resp)?)
    }
}

// ---- MockRpc: scripted responses for host tests ----

/// A mock `Rpc` that returns scripted raw JSON-RPC responses in FIFO order.
/// Each trait call consumes one response and runs it through the same parse
/// layer as `WakiRpc`, so the full response→typed path is exercised. Returns
/// `RpcError::Unexpected` once the queue is empty.
pub struct MockRpc {
    responses: RefCell<VecDeque<Value>>,
}

impl MockRpc {
    pub fn new(responses: Vec<Value>) -> Self {
        Self { responses: RefCell::new(responses.into()) }
    }
    fn next_response(&self) -> Result<Value, RpcError> {
        self.responses
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| RpcError::Unexpected("MockRpc: no more scripted responses".to_string()))
    }
}

impl Rpc for MockRpc {
    fn get_latest_blockhash(&self) -> Result<BlockhashInfo, RpcError> {
        let resp = self.next_response()?;
        parse_blockhash(rpc_result(&resp)?)
    }
    fn get_account_info(&self, _pubkey: &Pubkey) -> Result<Option<AccountInfo>, RpcError> {
        let resp = self.next_response()?;
        parse_account_info(rpc_result(&resp)?)
    }
    fn get_signatures_for_address(&self, _pubkey: &Pubkey, _limit: usize) -> Result<Vec<TxSummary>, RpcError> {
        let resp = self.next_response()?;
        parse_signatures(rpc_result(&resp)?)
    }
    fn send_transaction(&self, _tx: &[u8]) -> Result<String, RpcError> {
        let resp = self.next_response()?;
        parse_send_tx(rpc_result(&resp)?)
    }
}