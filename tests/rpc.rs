//! Host tests for the `rpc` module (slice 6 of palinurus-core).
//!
//! The JSON-RPC 2.0 request-building and response-parsing logic is pure and
//! fully host-testable via `MockRpc` (scripted raw JSON-RPC responses → typed
//! Rust). `WakiRpc` (the waki transport) shares the same parse layer and is
//! only exercised live inside a wasm32-wasip2 component with wasi:http — not
//! run on host. No live network in `cargo test`.

use palinurus_core::base58::Pubkey;
use palinurus_core::rpc::{
    rpc_request, rpc_result, parse_account_info, parse_blockhash, parse_send_tx,
    parse_signatures, MockRpc, Rpc, RpcError, WakiRpc,
};
use base64::prelude::{BASE64_STANDARD, Engine as _};

fn pk(b: u8) -> Pubkey { Pubkey::from_bytes([b; 32]) }

// ---- JSON-RPC request envelope ----

#[test]
fn rpc_request_envelope_is_correct() {
    let req = rpc_request(7, "getLatestBlockhash", serde_json::json!([]));
    assert_eq!(req["jsonrpc"], "2.0");
    assert_eq!(req["id"], 7);
    assert_eq!(req["method"], "getLatestBlockhash");
    assert_eq!(req["params"], serde_json::json!([]));

    let req2 = rpc_request(3, "getAccountInfo", serde_json::json!(["11111111111111111111111111111111", {"encoding":"base64"}]));
    assert_eq!(req2["method"], "getAccountInfo");
    assert_eq!(req2["params"][0], "11111111111111111111111111111111");
    assert_eq!(req2["params"][1]["encoding"], "base64");
}

// ---- response parsing: getLatestBlockhash ----

#[test]
fn parse_blockhash_extracts_value() {
    // blockhash = system program id (32 zeros) as a stand-in valid base58 32-byte value.
    let resp = serde_json::json!({
        "jsonrpc":"2.0","id":1,
        "result":{"context":{"slot":123},"value":{"blockhash":"11111111111111111111111111111111","lastValidBlockHeight":9999}}
    });
    let info = parse_blockhash(rpc_result(&resp).unwrap()).unwrap();
    assert_eq!(info.blockhash, [0u8; 32]);
    assert_eq!(info.last_valid_block_height, 9999);
}

#[test]
fn parse_blockhash_rejects_missing_field() {
    let resp = serde_json::json!({"jsonrpc":"2.0","id":1,"result":{"value":{}}});
    let err = parse_blockhash(rpc_result(&resp).unwrap()).unwrap_err();
    assert!(matches!(err, RpcError::Unexpected(_)), "got {err:?}");
}

// ---- response parsing: getAccountInfo ----

#[test]
fn parse_account_info_null_is_none() {
    let resp = serde_json::json!({"jsonrpc":"2.0","id":1,"result":{"context":{"slot":1},"value":null}});
    assert_eq!(parse_account_info(rpc_result(&resp).unwrap()).unwrap(), None);
}

#[test]
fn parse_account_info_decodes_base64_data() {
    let owner = Pubkey::SYSTEM;
    let payload = b"hello nonce world";
    let b64 = BASE64_STANDARD.encode(payload);
    let resp = serde_json::json!({
        "jsonrpc":"2.0","id":1,
        "result":{"context":{"slot":5},"value":{
            "data":[b64, "base64"],
            "owner": owner.to_string(),
            "lamports": 1234567,
            "executable": false,
            "rentEpoch": 200
        }}
    });
    let ai = parse_account_info(rpc_result(&resp).unwrap()).unwrap().expect("Some");
    assert_eq!(ai.data, payload);
    assert_eq!(ai.owner, owner);
    assert_eq!(ai.lamports, 1234567);
    assert!(!ai.executable);
}

// ---- response parsing: getSignaturesForAddress ----

#[test]
fn parse_signatures_extracts_list() {
    let resp = serde_json::json!({
        "jsonrpc":"2.0","id":1,
        "result":[
            {"signature":"5Tx...abc","err":null,"slot":100,"blockTime":1753000000},
            {"signature":"9Tx...def","err":"InstructionError","slot":101,"blockTime":null}
        ]
    });
    let sigs = parse_signatures(rpc_result(&resp).unwrap()).unwrap();
    assert_eq!(sigs.len(), 2);
    assert_eq!(sigs[0].signature, "5Tx...abc");
    assert_eq!(sigs[0].err, None);
    assert_eq!(sigs[0].slot, 100);
    assert_eq!(sigs[0].block_time, Some(1753000000));
    assert_eq!(sigs[1].err.as_deref(), Some("InstructionError"));
    assert_eq!(sigs[1].block_time, None);
}

// ---- response parsing: sendTransaction ----

#[test]
fn parse_send_tx_returns_signature() {
    let resp = serde_json::json!({"jsonrpc":"2.0","id":1,"result":"5Sig...xyz"});
    assert_eq!(parse_send_tx(rpc_result(&resp).unwrap()).unwrap(), "5Sig...xyz");
}

// ---- JSON-RPC error object ----

#[test]
fn rpc_result_surfaces_jsonrpc_error_object() {
    let resp = serde_json::json!({
        "jsonrpc":"2.0","id":1,
        "error":{"code":-32000,"message":"Transaction simulation failed"}
    });
    let err = rpc_result(&resp).unwrap_err();
    match err {
        RpcError::Rpc { code, message } => {
            assert_eq!(code, -32000);
            assert!(message.contains("Transaction simulation"));
        }
        other => panic!("expected RpcError::Rpc, got {other:?}"),
    }
}

#[test]
fn rpc_result_rejects_malformed_envelope() {
    let resp = serde_json::json!({"foo":"bar"});
    assert!(matches!(rpc_result(&resp), Err(RpcError::Unexpected(_))));
}

// ---- MockRpc end-to-end (response → parse → typed) ----

#[test]
fn mock_rpc_get_latest_blockhash_end_to_end() {
    let mock = MockRpc::new(vec![
        serde_json::json!({"jsonrpc":"2.0","id":1,"result":{"context":{"slot":1},"value":{"blockhash":"11111111111111111111111111111111","lastValidBlockHeight":42}}}),
    ]);
    let info = mock.get_latest_blockhash().unwrap();
    assert_eq!(info.blockhash, [0u8; 32]);
    assert_eq!(info.last_valid_block_height, 42);
}

#[test]
fn mock_rpc_get_account_info_then_parse_nonce_account() {
    // Integration with slice 5: getAccountInfo on a nonce account → base64-decode →
    // parse_nonce_account. Build the nonce account data via the solana_program oracle.
    use palinurus_core::durable_nonce::{parse_nonce_account, NonceState, NonceVersion};
    use solana_program::nonce::state::{DurableNonce, State, Versions};
    let authority = pk(0x20);
    let blockhash = solana_program::hash::Hash::new_from_array([0x11; 32]);
    let durable_nonce = DurableNonce::from_blockhash(&blockhash);
    let expected_nonce_bytes = durable_nonce.as_hash().to_bytes();
    let versions = Versions::new(State::new_initialized(
        &solana_program::pubkey::Pubkey::new_from_array(authority.to_bytes()),
        durable_nonce,
        5000,
    ));
    let raw = bincode::serialize(&versions).unwrap();
    let b64 = BASE64_STANDARD.encode(&raw);

    let mock = MockRpc::new(vec![
        serde_json::json!({"jsonrpc":"2.0","id":1,"result":{"context":{"slot":1},"value":{
            "data":[b64, "base64"], "owner": Pubkey::SYSTEM.to_string(),
            "lamports": 1000000, "executable": false, "rentEpoch": 0
        }}}),
    ]);
    let ai = mock.get_account_info(&pk(0x10)).unwrap().expect("Some");
    let nonce = parse_nonce_account(&ai.data).unwrap();
    assert!(matches!(nonce.version, NonceVersion::Current));
    match nonce.state {
        NonceState::Initialized(d) => {
            assert_eq!(d.authority, authority);
            assert_eq!(d.durable_nonce, expected_nonce_bytes);
            assert_eq!(d.lamports_per_signature, 5000);
        }
        NonceState::Uninitialized => panic!("expected Initialized"),
    }
}

#[test]
fn mock_rpc_get_signatures_end_to_end() {
    let mock = MockRpc::new(vec![
        serde_json::json!({"jsonrpc":"2.0","id":1,"result":[
            {"signature":"sig1","err":null,"slot":10,"blockTime":100},
            {"signature":"sig2","err":null,"slot":11,"blockTime":101}
        ]}),
    ]);
    let sigs = mock.get_signatures_for_address(&pk(0x10), 2).unwrap();
    assert_eq!(sigs.len(), 2);
    assert_eq!(sigs[0].signature, "sig1");
    assert_eq!(sigs[1].slot, 11);
}

#[test]
fn mock_rpc_send_transaction_end_to_end() {
    let mock = MockRpc::new(vec![
        serde_json::json!({"jsonrpc":"2.0","id":1,"result":"finalsig"}),
    ]);
    let sig = mock.send_transaction(&[0x00, 0x80, 0x01]).unwrap();
    assert_eq!(sig, "finalsig");
}

#[test]
fn mock_rpc_returns_error_when_queue_empty() {
    let mock = MockRpc::new(vec![]);
    assert!(matches!(mock.get_latest_blockhash(), Err(RpcError::Unexpected(_))));
}

#[test]
fn mock_rpc_calls_consume_responses_in_order() {
    let mock = MockRpc::new(vec![
        serde_json::json!({"jsonrpc":"2.0","id":1,"result":{"context":{"slot":1},"value":{"blockhash":"11111111111111111111111111111111","lastValidBlockHeight":1}}}),
        serde_json::json!({"jsonrpc":"2.0","id":2,"result":"sig2"}),
    ]);
    let _ = mock.get_latest_blockhash().unwrap();
    let sig = mock.send_transaction(&[]).unwrap();
    assert_eq!(sig, "sig2");
    // third call → empty
    assert!(mock.get_latest_blockhash().is_err());
}

// ---- WakiRpc construction (no network) ----

#[test]
fn waki_rpc_constructs_with_endpoint_and_optional_key() {
    let r = WakiRpc::new("https://api.devnet.solana.com".to_string(), None);
    assert_eq!(r.endpoint(), "https://api.devnet.solana.com");
    assert!(r.api_key().is_none());

    let r2 = WakiRpc::new("https://my-rpc.com/special-path".to_string(), Some("secretkey123".to_string()));
    assert_eq!(r2.endpoint(), "https://my-rpc.com/special-path");
    assert_eq!(r2.api_key().map(|s| s.to_string()), Some("secretkey123".to_string()));
}