# Palinurus

**The Solana DePIN node that talks.** A navigator at the physical edge, attesting back to the chain.

Palinurus is a suite of Solana-native tool plugins for [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw) — the self-hosted, Rust-based AI agent runtime (32k ⭐) — built as `wasm32-wasip2` WIT components against the ZeroClaw plugin contract. It brings real Solana capability to an autonomous agent that runs on your own hardware, with your own keys.

> Built for the [Superteam Brasil × ZeroClaw bounty](https://superteam.fun/earn/listing/zeroclaw) — *"Build Solana-native plugins for Zeroclaw 🦞"*. Track C (DePIN & the physical edge), the sponsor's favorite: *"the one nobody else can build."*

## Why

ZeroClaw runs on a Raspberry Pi with GPIO/I2C/SPI and an SOP engine triggered by MQTT and by peripherals. It's already a DePIN device — it just has no chain. Palinurus is the bridge: the navigator that takes a reading from the physical edge and commits a verifiable attestation back to Solana. A \$40 Pi becomes a Solana-reporting device.

The thesis: an agent with a private key and an LLM in the loop is a hot wallet with a prompt-injection surface. Palinurus treats custody as a first-class engineering problem — the agent proposes, a human or multisig disposes, and signing (when it happens at all) is scoped to a session key holding cents and constrained to a strict program allowlist.

## What's here

| Crate | What it does | Custody |
|---|---|---|
| `palinurus-core` | The shared `wasm32-wasip2`-friendly Solana substrate the plugins import: PDA derivation, base58, borsh, versioned-tx construction, durable-nonce handling, RPC over the host's `wasi:http`, response shaping. Pure-core/thin-shim, host-tested, MIT, crates.io-published. | — |
| `depin-attest` *(planned)* | Take a sensor reading (from the host's hardware via the SOP engine), commit a periodic attestation on-chain via the [Solana Attestation Service](https://attest.solana.com/) (or memo fallback), with a nonce-based replay guard. Ships with a wiring diagram. | T1 (unsigned, multisig signs) default + T2 (autonomous, scoped session key) opt-in |
| `depin-rewards` *(planned)* | Watch a DePIN hotspot (Helium / Hivemapper) for rewards, uptime, downtime; fire Telegram/Discord alerts. Build an unsigned rewards-claim tx a human signs. | T0 (reads + alerts) + T1 (unsigned claim tx) |

> A stream of signed attestations from a stable key *is* an oracle feed — the `depin-attest` README documents how to consume the attestation stream as an oracle, rather than shipping a separate `oracle-publish` component. Depth over breadth, per the bounty's guidance.

## Status

🚧 **In development.** Currently de-risking the hardest piece — PDA derivation inside the WASM sandbox (`solana-sdk`/`solana-program` cannot be compiled for `wasm32-wasip2`; we hand-roll `find_program_address` from `sha2` + `curve25519-dalek`). See `src/pda.rs`.

## Build & test

```bash
rustup target add wasm32-wasip2
cargo test                                  # host tests, no wasm toolchain needed
cargo build --release --target wasm32-wasip2 # confirm the core compiles to the component target
```

## License

MIT. Palinurus is named for *Palinurus* — the spiny-lobster genus and Virgil's helmsman-navigator in the *Aeneid*, who reported from beyond the edge.