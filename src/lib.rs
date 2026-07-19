//! Palinurus — minimal wasm32-wasip2-friendly Solana substrate for ZeroClaw WIT tool plugins.
//!
//! Track C (DePIN) core for the Superteam Brasil × ZeroClaw bounty. MIT-licensed.
//! Pure logic, no wasm dependency — host-testable with `cargo test`; the plugins
//! import this as a crates.io dependency and compile it into their `wasm32-wasip2`
//! components.
//!
//! `solana-sdk` / `solana-program` cannot be compiled inside a WIT component (they
//! pull in syscall stubs that don't build for `wasm32-wasip2`). This crate hand-rolls
//! only the primitives a Solana tool plugin needs:
//!   - PDA derivation (`pda::find_program_address`) — reimplementation of
//!     `solana_program::Pubkey::find_program_address` using `sha2` + `curve25519-dalek`.
//!   - (more modules landing as the build progresses: base58, borsh helpers,
//!     versioned-tx construction, durable-nonce handling, RPC over the host's
//!     `wasi:http`, response shaping to ~200 tokens).

pub mod base58;
pub mod borsh;
pub mod pda;
pub mod response;
pub mod rpc;
pub mod durable_nonce;
pub mod versioned_tx;

pub use base58::Pubkey;
pub use borsh::{memo_ix_data, CreateAttestationIxData};
pub use pda::{find_program_address, is_on_curve};
pub use response::{estimate_tokens, shape_account_info, shape_attestation_result, shape_signatures, short_pubkey, AccountKind};
pub use rpc::{parse_account_info, parse_blockhash, parse_send_tx, parse_signatures, rpc_request, rpc_result, MockRpc, Rpc, RpcError, WakiRpc};
pub use durable_nonce::{build_with_durable_nonce, nonce_advance_ix, nonce_authorize_ix, parse_nonce_account, NonceAccount, NonceData, NonceError, NonceState, NonceVersion};
pub use versioned_tx::{build_unsigned, serialize as serialize_tx, AccountMeta, Instruction, MessageHeader, MessageV0, VersionedTransaction};