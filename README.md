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
| `depin-attest` | Take a sensor reading (from the host's hardware via the SOP engine), commit a periodic attestation on-chain via the [Solana Attestation Service](https://attest.solana.com/) (or memo fallback), with a durable-nonce replay guard. 68 host tests, wasm clean. | T1 (unsigned, multisig signs) default + T2 (autonomous, scoped session key) opt-in |
| `depin-rewards` | Watch a public Helium hotspot (no ownership required) for online/offline flips + rewards; fire real Telegram alerts; draft an unsigned rewards-claim tx. 49 host tests, wasm clean, **no signing key anywhere in the crate**. | T0 (reads + alerts) + T1 (unsigned claim, roadmap). No T2 |

> A stream of signed attestations from a stable key *is* an oracle feed — the `depin-attest` README documents how to consume the attestation stream as an oracle, rather than shipping a separate `oracle-publish` component. Depth over breadth, per the bounty's guidance.

## Status

✅ **Phase 0–3 complete.** `palinurus-core` v0.1.0 is live on crates.io (PDA derivation hand-rolled from `sha2` + `curve25519-dalek` — `solana-sdk`/`solana-program` can't compile for `wasm32-wasip2`, so we rebuilt `find_program_address` and cross-checked it byte-for-byte against `solana_program` and `@solana/web3.js`). Both plugins are implemented and live on [PR #76](https://github.com/zeroclaw-labs/zeroclaw-plugins/pull/76) to `zeroclaw-labs/zeroclaw-plugins` — 188 host tests across the trio, all `clippy -D warnings` + `wasm32-wasip2` clean.

- **`depin-rewards` rewards path is verified live** against the real Relay API on the free Community tier (the live smoke test surfaced & fixed 3 real bugs the mocked tests couldn't catch — see the plugin README).
- **`depin-rewards` claim tx is deferred** by design: Helium hotspots are compressed NFTs, so the claim is `distribute_compression_rewards_v0` + a DAS `get_asset_proof` merkle proof — a focused multi-session effort, not a rushed slice. The homework is done; impl is the next milestone.

🚧 **Phase 4–5 in progress** — the demo track (wiring diagram ✅, marketing site, demo recording guide, ≤3 min demo video) + Superteam submission. Deadline: winner announced Aug 21 2026.

## Wiring

<img src="docs/wiring-diagram.svg" alt="Palinurus wiring diagram: physical edge → ZeroClaw agent → Solana, with custody tiers and the cold signing path" width="100%"/>

A $40 Raspberry Pi running ZeroClaw hosts two Palinurus WIT plugins. `depin-attest` turns a sensor reading into a Solana Attestation Service attestation (unsigned tx → human/multisig signs). `depin-rewards` watches any public Helium hotspot via the Relay API and fires Telegram alerts the moment it goes dark. The agent never holds a main wallet key — see the cold path across the bottom.

## Build & test

```bash
rustup target add wasm32-wasip2
cargo test                                  # host tests, no wasm toolchain needed
cargo build --release --target wasm32-wasip2 # confirm the core compiles to the component target
```

## License

MIT. Palinurus is named for *Palinurus* — the spiny-lobster genus and Virgil's helmsman-navigator in the *Aeneid*, who reported from beyond the edge.