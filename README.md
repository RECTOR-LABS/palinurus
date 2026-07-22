<div align="center">

# 🦞 Palinurus

**The Solana DePIN node that talks.** A navigator at the physical edge, attesting back to the chain.

*A ZeroClaw agent on a \$40 Raspberry Pi becomes a Solana-attesting, reward-watching DePIN node — the agent proposes, a human/Squads multisig disposes, no main key ever leaves the cold path.*

[![Track](https://img.shields.io/badge/Track-C_·_DePIN-6f42c1)](https://superteam.fun/earn/listing/zeroclaw)
[![Custody](https://img.shields.io/badge/custody-T0_·_T1_·_T2-0969da)](#safety--custody)
[![Tests](https://img.shields.io/badge/tests-212_host-brightgreen)](#code-quality)
[![crates.io](https://img.shields.io/crates/v/palinurus-core?label=palinurus%20core&color=orange)](https://crates.io/crates/palinurus-core)
[![devnet](https://img.shields.io/badge/devnet-T2_verified-brightgreen)](#live-on-devnet)
[![license](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![site](https://img.shields.io/badge/site-live-blue)](https://palinurus.rectorspace.com)

</div>

<div align="center">

<img src="docs/wiring-diagram.svg" alt="Palinurus wiring: physical edge → ZeroClaw agent → Solana, custody tiers + cold signing path" width="100%"/>

</div>

> Built for the [Superteam Brasil × ZeroClaw bounty](https://superteam.fun/earn/listing/zeroclaw) — *"Build Solana-native plugins for Zeroclaw 🦞"*. **Track C (DePIN & the physical edge)**, the sponsor's favorite: *"the one nobody else can build."*

Palinurus is a suite of Solana-native tool plugins for [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw) — the self-hosted, Rust-based AI agent runtime (32k ⭐) — built as `wasm32-wasip2` WIT components against the ZeroClaw plugin contract. It brings real Solana capability to an autonomous agent that runs on your own hardware, with your own keys.

---

## Why — the problem Palinurus solves

**ZeroClaw is already a DePIN device — it just has no chain.** It runs on a \$40 Raspberry Pi with GPIO/I2C/SPI and an SOP engine triggered by MQTT and by peripherals. That's a node at the physical edge. What's missing is the *navigator*: something that takes a reading and commits a verifiable attestation back to Solana — turning a Pi into a chain-reporting device.

**The hard part isn't signing — it's signing *safely*.** The naive design hands the agent a private key and lets the LLM decide what to sign. But an agent with a private key and an LLM in the loop *is a hot wallet with a prompt-injection surface* — one crafted message away from draining itself. The real engineering question is whether the agent can act on-chain at all while *guaranteed* failing closed under injection, never moving value it shouldn't, and never exposing a main key.

**Palinurus treats custody as a first-class engineering problem.** The agent proposes; a human or Squads multisig disposes; and signing, when it happens at all, is scoped to a session key holding cents and constrained to a strict program allowlist. Two plugins cover the two faces of DePIN on Solana — *attesting* (the edge reports to the chain) and *reward-watching* (the chain reports back to the owner) — on a shared `wasm32-wasip2` substrate.

---

## How this maps to the judging rubric

A judge should be able to score each criterion without hunting. The body is ordered by rubric weight.

| Criterion | Weight | One-line evidence | Section |
|---|---:|---|---|
| **Real utility** | 30% | `depin-rewards` = the daily-use plugin a stranger runs for months (no hardware, real Capybara alerts); `depin-attest` = reference impl **verified live on devnet**; `claim_tx` deferred honestly, not faked | [→ Real utility](#real-utility) |
| **Safety & custody** | 25% | Full **T0 + T1 + T2** custody tiers; **4-vector fail-closed** prompt-injection transcripts (test-backed, both plugins); **no signing key anywhere** in `depin-rewards` (a test asserts it); devnet T2 guards enforced **before signing** | [→ Safety & custody](#safety--custody) |
| **Code quality** | 20% | Pure-core / thin-shim split; **212 host tests** (71 core + 83 attest + 58 rewards); `palinurus-core` **published on crates.io**; TS oracle byte-for-byte cross-checks; `clippy -D warnings` + `wasm32-wasip2` clean (both crates, both modes) | [→ Code quality](#code-quality) |
| **Merge-readiness** | 15% | Matches `redact-text` layout; minimal permissions (`http_client` + `config_read`); v0.1.0 semver, kebab names; **both WIT shims wired** (incl. the `depin-rewards` `execute()` dispatch fix — was a stub, now routes to the pure core + `WakiHttp`); crates.io dep (no fork URL) | [→ Merge-readiness](#merge-readiness) |
| **Demo & docs** | 10% | 3 screenshots (explorer tx + terminal T2 + marketing site); wiring diagram (SVG); recording guide + chunk-by-chunk walkthrough; **live marketing site**; one-command demo drivers | [→ Demo & docs](#demo--docs) |

---

## Table of contents

- [Real utility (30%)](#real-utility) — would a stranger install this and run it for a month?
- [Safety & custody (25%)](#safety--custody) — can we prompt-inject it? Fail closed? Tier honest?
- [Security audit](#security-audit) — auditor-grade review of the T2 path (no critical vuln; closed loop)
- [Code quality (20%)](#code-quality) — pure core, real tests, clean shim, idiomatic Rust
- [Merge-readiness (15%)](#merge-readiness) — could upstream merge it today?
- [Demo & docs (10%)](#demo--docs) — understand and run in 5 minutes?
- [Build & test](#build--test)
- [License](#license)

---

## Real utility

*Rubric: "Would a stranger install this and still be running it in a month?"*

Two plugins, not one — and the second is the one a stranger actually installs daily.

### `depin-rewards` — the daily-useful plugin (no hardware, no hotspot ownership)

The *"stranger installs this and runs it for a month"* plugin. Watches **any public Helium/Hivemapper-class hotspot** via the public Relay API (Helium-Foundation-sponsored, free Community tier — within the bounty's granted "read access to any aggregator API") and fires **real Telegram alerts** the moment it goes dark. **No Raspberry Pi, no hotspot ownership required** — a stranger with a free Relay key + a free Telegram bot token is running it today.

- ✅ `status` — hotspot online/offline + owner + location + maker now.
- ✅ `summary` — daily rewards total + beacon/witness/dc breakdown (Relay reward-shares list endpoint, client-side sum — the `/totals` aggregate is 401-gated on the free tier; verified live).
- ✅ `watch` — the cron workhorse: online→offline flips → instant Telegram alert + optional 08:00 daily rewards summary.
- 📋 `claim_tx` — **deferred honestly, not faked.** Helium hotspots are compressed NFTs (cNFTs), so the correct claim ix is `distribute_compression_rewards_v0` + a DAS `get_asset_proof` merkle proof — not the regular `distribute_rewards_v0`. The design is verified (program id, PDAs, custody posture all documented in the plugin README); impl is the next focused milestone. We shipped the alerts core complete + correct rather than rush a half-verified claim tx.

> **Live-verified:** the rewards path ran against the real Relay API on the free Community tier and surfaced + fixed **3 real bugs** the mocked tests couldn't catch (see `depin-rewards` README). The demo shows **0.02 HNT** earned by the Capybara test hotspot over a real window.

### `depin-attest` — the reference impl, verified on devnet

A \$40 Pi running ZeroClaw becomes a Solana-attesting DePIN node: the agent takes a sensor reading, the plugin builds a versioned tx with a [Solana Attestation Service](https://attest.solana.com/) `create_attestation` (or memo fallback) + a **durable nonce** (the blockhash-expiry fix — a queued tx doesn't die), and returns a ~200-token summary. **T1 default** (unsigned — a human/multisig signs) + **T2 opt-in** (scoped session key). The T2 path is **verified live on devnet** — a real, explorer-verifiable on-chain attestation. See [Live on devnet](#live-on-devnet) below (it's also Safety evidence — the custody guards fired pre-sign on a real submission).

> A stream of signed attestations from a stable key *is* an oracle feed — the `depin-attest` README documents how to consume the attestation stream as an oracle, rather than shipping a separate `oracle-publish` component. **Depth over breadth**, per the bounty's guidance.

### Custody at a glance

| Plugin | Reads (T0) | Unsigned tx (T1) | Autonomous sign (T2) |
|---|---|---|---|
| `depin-attest` | — | ✅ durable-nonce attestation | ✅ opt-in, scoped session key, allowlist + caps — **verified on devnet** |
| `depin-rewards` | ✅ Relay + Telegram | 📋 unsigned claim tx (design documented, deferred) | ❌ never (claim moves value → multisig) |

The agent never holds a main wallet key. Pattern: *agent proposes, multisig disposes.*

---

## Safety & custody

*Rubric: "Can we prompt-inject it? Does it fail closed? Is the tier honest?"*

Custody is treated as a first-class engineering problem — the thesis is that an agent with a private key + an LLM in the loop **is a hot wallet with a prompt-injection surface**. Every plugin declares + defends its tier with a **fail-closed prompt-injection transcript backed by host tests**.

### `depin-rewards` — T0/T1 only, no signing key anywhere

**The plugin holds no key of any kind.** Not a main wallet key, not a session key, not a fee-payer key. There is no `ed25519` / signing dependency and no signing code path anywhere in the crate — verified by `no_signing_capabilities_in_crate`, a test that greps `Cargo.toml` + source for signing tokens and asserts none. Four attack vectors, four rejections:

| Vector | Guard (test-backed) | Result |
|---|---|---|
| Watch/alert an unconfigured hotspot | `enforce_hotspot_allowlist` (wired into every action) | `Err::Config("hotspot 'evil-id' not in configured allowlist")` |
| Exfiltrate `relay_api_key` / `telegram_bot_token` via output | redacting `Debug` impl; shapers never echo credentials | sentinel strings absent from output + Debug |
| Redirect the Telegram alert to an attacker's chat | `chat_id` sourced from **config**, never the message text | recorded POST `chat_id` == configured; `"666"` ignored |
| Claim rewards for a different owner (`claim_tx`, roadmap) | owner sourced from Relay `get-hotspot`, never the message | no message-supplied owner parameter by construction |

**Worst-case blast radius of a prompt injection:** Telegram spam to the *configured* chat (rate-limited by `watch` cadence) or a claim tx drafted for the *configured* hotspot's *own* owner. Nuisance, not theft.

### `depin-attest` — full T2 guards, fail-closed, proven on devnet

T2 autonomous signing is the brutal bar: *if a judge can prompt-inject the agent into draining the session key → zero on safety regardless of code quality.* The T2 path enforces, **before signing**:

- **Program allowlist `{System, SAS, Memo}`** — a value-transfer instruction is not expressible; `enforce_program_allowlist` checks every ix.
- **Session-key identity** — the signing key must equal `authority` = `payer` = `nonce_authority` (one scoped identity, enforced).
- **Hard caps** — per-tx fee cap (`max_lamports_per_tx`) + per-day attestation cap (`max_attestations_per_day`, UTC day from the reading timestamp).
- **No main wallet key** — the session key is dedicated, scoped, cents-only.

Four attack vectors, four rejections (full transcript in the `depin-attest` README): memo-encoded "transfer 1 SOL" (inert — no transfer code path) · injected SPL Token ix (fail closed — not in allowlist) · "print the session key" (key never serialized into any output; `Debug` redacted) · daily-cap bypass via timestamp rolling (fail closed at cap+1; rolled timestamps produce different attestation PDAs — natural dedup).

### Live on devnet — the T2 proof (also Real-utility evidence)

The `depin-attest` T2 custody path is **verified on Solana devnet** — a real, explorer-verifiable attestation, not an unsigned draft. This is the **only on-chain-verified entry** in the Track-C field.

<p align="center"><em>The T2 demo driver — signs + submits on camera:</em></p>

<img src="docs/screenshots/terminal-t2-attestation.png" alt="Terminal: depin-attest T2 demo driver output — real signature + explorer link" width="88%"/>

<p align="center"><em>The confirmed transaction on Solana explorer (Success · Finalized):</em></p>

<img src="docs/screenshots/devnet-attestation-explorer.png" alt="Solana explorer: confirmed devnet attestation — Success, Finalized" width="88%"/>

- tx [`BsdBnMtJ…2qGYo`](https://explorer.solana.com/tx/BsdBnMtJHFarDREDdA7hgbGp2hUe4mBMzw4erjM9jrVhQdRyS4yQxtTpCEtobrxjiCj5zKMHMuwVKPd2Pv2qGYo?cluster=devnet) · **Success / Finalized** · slot `477808575` · fee `0.000005` SOL · version `0` (versioned tx → durable nonce)
- durable nonce advanced on this run (replay guard live) · reading committed on-chain as a memo: `palinurus: bme280-1=24.7celsius @ 1784621332`
- custody guards enforced **before signing**: session-key identity · program allowlist `{System, SAS, Memo}` · lamport cap · daily cap

> **SAS vs memo path.** The default is the **memo program** (cheap, high-throughput — the landed proof above). The **SAS** path (`create_attestation`, verifiable + credential-bound) builds the instruction + derives the PDA, but on-chain landing is blocked on schema creation (SAS `0x4` — a stale-arg issue against `sas-lib@1.0.10`'s `getCreateSchemaInstruction`; the credential creates cleanly). SAS ix + PDA are tested + cross-checked via TS oracles. On-chain SAS landing is the next milestone.

---

## Security audit

The T2 autonomous-sign path is money-critical, so it gets an auditor-grade review — not just claims. The full report: [`docs/security-audit.md`](docs/security-audit.md) (read-only, cite `file:line` for every finding, skeptical posture; methodology adapted from our [`solana-cpi-safety-skill`](https://github.com/RECTOR-LABS/solana-cpi-safety-skill) auditor).

**Result: no critical vulnerability.** The one foundational trust assumption — that the host injects the plugin's `__config` and strips any LLM-supplied one — is **confirmed safe by the ZeroClaw host contract** (`zeroclaw/crates/zeroclaw-plugins/src/runtime.rs:157-168`: `obj.remove("__config")` then re-inject the trusted config; host-tested). Every other finding was followed up (a closed audit loop):

| Finding | Disposition |
|---|---|
| F2 — System allowlist was program-level (a `System::Transfer` could pass the guard) | ✅ Hardened to **instruction-level** — only `AdvanceNonceAccount` (disc `0x04`); value transfer is now unexpressible at the *guard* |
| F5 — unbounded memo length | ✅ Capped at 566 B, validated before any tx build |
| F6 — nonce account owner unchecked | ✅ `owner == System` verified before parse |
| F4 — session key not zeroized | ✅ Raw key bytes zeroized after `SigningKey::from_bytes` |
| F3 — daily cap is a rate-hint, not a guard (timestamp-rollable) | ✅ Disclosure sharpened; on-chain counter PDA roadmaped |
| F7 — unreachable `expect()` panics | ✅ Accepted w/ rationale (infallible); core `Result` variant roadmaped |

Plus 6 passing notes (signing correctness **devnet-confirmed**, identity guard pre-sign, durable-nonce replay guard, input validation, redacted `Debug`, signed-tx never surfaced to the LLM). **83 attest tests** (was 78; +5 audit-driven), clippy + wasm clean.

---

## Code quality

*Rubric: "Pure core, real tests, clean shim, idiomatic Rust."*

**Pure-core / thin-shim split.** Every plugin puts its logic in a `src/<name>.rs` pure module with **no wasm deps** (host-testable with `cargo test`), and a `src/lib.rs` thin `#[cfg(target_family = "wasm")]` component shim that parses args, builds config, calls the pure core, and shapes a ~200-token `ToolResult`. The consensus-critical primitives live in `palinurus-core`.

| Crate | What | Tests | wasm | clippy |
|---|---|---:|:---:|:---:|
| `palinurus-core` | PDA derivation (hand-rolled `sha2` + `curve25519-dalek`), base58, borsh, versioned-tx, durable-nonce, JSON-RPC over `waki`, response shaping | **71** | ✅ | ✅ `-D warnings` |
| `depin-attest` | Sensor reading → SAS/memo attestation, durable-nonce, T1+T2 custody | **83** (77 core + 6 demo) | ✅ | ✅ |
| `depin-rewards` | Relay reads + Telegram alerts, T0/T1 custody, no signing key | **58** (55 core + 3 demo) | ✅ | ✅ |
| | | **212** | | |

**Why hand-roll the substrate?** `solana-sdk` / `solana-program` can't compile inside a WIT component (syscall stubs). PDA derivation is rebuilt from `sha2` + `curve25519-dalek` and **cross-checked byte-for-byte against `solana_program` and `@solana/web3.js`** (TS oracles, host dev-deps) — the layout gotcha (`sha256(seeds ‖ bump ‖ program_id ‖ "ProgramDerivedAddress")`, bump as the last seed) was caught by the oracle, not guessed. Both plugins depend on the **published crates.io** core (`palinurus-core = "0.1"`) — no fork git URL for upstream reviewers.

**Output shaping.** Every tool returns ≤200 tokens / ≤800 chars — never a raw 40KB `getProgramAccounts` dump (the bounty's context-window trap). Judges can `execute` and count tokens.

---

## Merge-readiness

*Rubric: "Manifest, docs, versioning, permissions minimal, could upstream merge it today?"*

- **Layout matches `redact-text` exactly** (hard req #1) — `src/<name>.rs` (pure) + `src/lib.rs` (thin shim) + `tests/` + `manifest.toml`. MIT licensed.
- **Minimal permissions** — each `manifest.toml` declares only what the plugin needs: `http_client` (Relay/RPC) + `config_read` (the jailed config section). Nothing broader.
- **Versioning** — `v0.1.0` semver; kebab-case crate names (`depin-attest`, `depin-rewards`); `palinurus-core v0.1.0` live on crates.io.
- **Crates.io dep, not a fork** — `palinurus-core = "0.1"` resolves from the registry; no git URL, no path dep for upstream reviewers to chase.
- **Both WIT shims are wired.** `depin-attest`'s shim has shipped wired since slice A. `depin-rewards`'s `execute()` **was** a slice-A stub returning "not wired yet" — caught in the pre-submission judge audit and **fixed**: it now dispatches to the pure core via `execute_entry` + `RewardsRequest` (3 TDD dispatch tests) + a real wasm `WakiHttp` (waki-backed `HttpClient` impl, `cfg(target_family="wasm")`). The plugin **functions as a WIT component** — `wasm32-wasip2` build is clean.

> The audit catch: a working pure core + demo driver is not enough — the actual WIT entry point must be wired too. The fix mirrors `depin-attest`'s architecture: an `execute_entry` dispatch seam (host-testable) + a wasm-only `WakiHttp` (verified by the wasm build).

---

## Demo & docs

*Rubric: "Can we understand and run it in 5 minutes?"*

### Screenshots (3)

| Terminal: T2 demo driver | Explorer: confirmed devnet tx | Marketing site |
|---|---|---|
| signs + submits on camera | Success · Finalized | palinurus.rectorspace.com |

See them inline above ([Live on devnet](#live-on-devnet)) and in the [Marketing site](#marketing-site) section.

### Wiring diagram

The 5-column flow above (physical edge → ZeroClaw agent → Solana) with custody tiers + the cold signing path across the bottom. Source: `docs/wiring-diagram.svg` (dark-mode, embeddable).

### One-command demo drivers

Two TDD'd host drivers behind `demo` features (off by default — the shipped wasm is clean):

```bash
# Rewards — no hardware, live Relay (chunks 2-5 of the demo):
cd plugins/depin-rewards && RELAY_API_KEY=<key> cargo run --features demo --bin palinurus-demo -- all

# Attest — the T2 on-camera beat (chunk 6), real devnet wallet:
cd plugins/depin-attest && ATTEST_MODE=t2 ATTEST_MEMO_FALLBACK=true \
  ATTEST_SESSION_KEYFILE=<path/to/solana-keypair.json> \
  cargo run --features demo --bin depin-attest-demo
```

### Marketing site

<p align="center">
<a href="https://palinurus.rectorspace.com"><img src="docs/screenshots/marketing-site.png" alt="Palinurus marketing site — palinurus.rectorspace.com" width="90%"/></a>
</p>

Live at **[palinurus.rectorspace.com](https://palinurus.rectorspace.com)** — Next.js 16 + Tailwind 4, dark mode, auto-deployed from `main` via Vercel (git connected, `rootDirectory=site`).

### Recording guide + chunk-by-chunk walkthrough

`docs/demo-recording-guide.md` (7-chunk shot list + VO script + ffmpeg cmds + risk register) and `docs/demo-recording-walkthrough.html` (per-chunk shot-by-shot + the verify gate). Chunk 6 is the T2 on-camera beat; chunk 4 is the live Telegram alert.

---

## Build & test

```bash
rustup target add wasm32-wasip2
cargo test                                  # 212 host tests, no wasm toolchain needed
cargo build --release --target wasm32-wasip2 # the core compiles to the component target
cargo clippy --all-targets -- -D warnings    # zero warnings, both crates, both feature modes
```

The plugin + demo driver source, the recording guide, and the full custody + injection transcripts live in the [PR #76](https://github.com/zeroclaw-labs/zeroclaw-plugins/pull/76) repo (`plugins/depin-attest`, `plugins/depin-rewards`).

## License

MIT. Palinurus is named for *Palinurus* — the spiny-lobster genus and Virgil's helmsman-navigator in the *Aeneid*, who reported from beyond the edge.