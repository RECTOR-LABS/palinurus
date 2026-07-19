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
pub mod pda;

pub use base58::Pubkey;
pub use pda::{find_program_address, is_on_curve};