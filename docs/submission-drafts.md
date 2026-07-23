# Palinurus — submission drafts (2026-07-23)

> Copy-paste-ready copy for the three remaining submission deliverables. All embed the
> Vimeo demo URL + PR #138 + the mainnet proof sig. Edit tone freely before posting.
>
> - **F1** = required bounty deliverable #2 — a showcase post in `#solana-bounty` (ZeroClaw Discord). Video ≤ 3 min.
> - **F2** = 3–4 build-in-public X posts (merge-enablement + maintenance commitment).
> - **One-pager** = optional Superteam submission attachment (drop the Vimeo + PR links in the form, attach this).

---

## Shared links (substitute `{{VIMEO}}` → the real URL)

- **Demo video (Vimeo, public):** https://vimeo.com/1212224147
- **PR #138 (submission link):** https://github.com/zeroclaw-labs/zeroclaw-plugins/pull/138
- **Mainnet proof (Finalized, Mainnet Beta):** https://explorer.solana.com/tx/YZTS16nNNrDhLhFHCtSMhcYTYAkcYQvPn2QUWfWvkw4bJxif77d1Ww36o3c4LYe6r69NAzYNJLDpz93DjR3G9TC
- **Marketing site:** https://palinurus.rectorspace.com
- **Core crate (crates.io):** https://crates.io/crates/palinurus-core
- **Repo:** https://github.com/RECTOR-LABS/palinurus
- **Bounty listing:** https://superteam.fun/earn/listing/zeroclaw

---

## F1 — Discord `#solana-bounty` showcase post (required deliverable #2)

> Paste below the video. Discord auto-embeds the Vimeo link as an inline player.
> Keep it under ~15 lines — maintaineurs skim. Bold the two on-camera beats.

---

**Palinurus — the Solana DePIN node that talks.** (Track C · Superteam Brasil × ZeroClaw bounty)

A pair of ZeroClaw WIT tool plugins (`wasm32-wasip2`) + a shared `palinurus-core` substrate, with a **real on-chain T2 attestation verified on mainnet** — not devnet, not a draft.

🎬 **Demo (2:03):** https://vimeo.com/1212224147
Two beats, both on camera:
  • a real Telegram alert fires the moment a watched Helium hotspot goes dark
  • the agent signs + submits a T2 attestation → **Finalized on Solana mainnet** (explorer flips to "Finalized (MAX Confirmations)" live)

🔗 **PR #138:** https://github.com/zeroclaw-labs/zeroclaw-plugins/pull/138
🔗 **Mainnet proof (Finalized, Mainnet Beta):** https://explorer.solana.com/tx/YZTS16nNNrDhLhFHCtSMhcYTYAkcYQvPn2QUWfWvkw4bJxif77d1Ww36o3c4LYe6r69NAzYNJLDpz93DjR3G9TC

**What it is:**
- **`depin-attest`** — sensor reading → Solana Attestation Service `create_attestation` (memo fallback) composed with a durable nonce (blockhash-expiry fix). T1 default (multisig signs) + T2 opt-in (scoped session key, `{System, SAS, Memo}` allowlist, caps) — **the T2 path is what's verified on mainnet above**.
- **`depin-rewards`** — watches any public Helium hotspot via Relay, fires Telegram alerts, drafts an unsigned claim tx. **No signing key anywhere in the crate** (a test asserts it).
- **`palinurus-core`** — the shared `wasm32-wasip2` substrate (PDA, base58, borsh, versioned-tx, durable-nonce, RPC over waki), **published on crates.io as v0.1.0** — both plugins depend on the published crate, no fork git URL.

**212 host tests**, clippy + wasm clean both crates. Fail-closed prompt-injection transcripts (test-backed) in each plugin README.

**Maintenance commitment:** `palinurus-core` is versioned on crates.io, so the plugins pin to a working version through any upstream WIT ABI change. I'll carry both plugins + the core through ZeroClaw's plugin-contract evolution — this isn't a hackathon one-shot, it's a maintained submission. Feedback on the custody posture + the claim-tx design (Helium cNFT compression path — deliberately deferred, documented) genuinely welcome, especially from anyone who's touched SAS or durable nonces. 🦞

---

## F2 — X build-in-public (3–4 posts)

> **POSTED LIVE 2026-07-23** via getpipher/browser skill (account @RZ1989sol).
> Thread root: https://x.com/RZ1989sol/status/2080165890998653114
> Each post ≤280 x-counted chars (269 / 277 / 243 / 266). Tone: dev-humble, lowercase, analogy-led (night-guard + toddler-with-credit-card). Below is the live copy (trimmed from the original drafts for length).

### 1/4 — the problem

Built something for the @superteambrasil × ZeroClaw bounty. Track C — DePIN & the physical edge.

ZeroClaw already runs on a $40 Raspberry Pi with GPIO/SOP engine. It's a DePIN device with no chain. The hard part isn't signing — it's signing *safely* with an LLM in the loop.

So we built Palinurus. 🧵

### 2/4 — the approach (custody first)

Palinurus treats custody as a first-class engineering problem:
• agent proposes, human/Squads multisig disposes — agent never holds a main key
• T2 autonomous signing is opt-in: scoped session key holding cents, {System, SAS, Memo} allowlist, hard caps
• a memo-encoded "transfer 1 SOL" is inert — no transfer code path
• fail-closed prompt-injection transcripts, test-backed

### 3/4 — the moat (PIN THIS)

The moat: the T2 path is verified on **Solana mainnet** — not devnet, not a draft.

sig `YZTS16nN…3G9TC` · Finalized · Mainnet Beta · 5000 lamports fee
https://explorer.solana.com/tx/YZTS16nNNrDhLhFHCtSMhcYTYAkcYQvPn2QUWfWvkw4bJxif77d1Ww36o3c4LYe6r69NAzYNJLDpz93DjR3G9TC

The guards held on a real submission. The only on-chain-verified entry in Track C. 🦞

### 4/4 — the demo + the commitment

2:03 of the real pipeline end-to-end, on camera: agent reads a Helium hotspot → Telegram alert the moment it goes dark → signs + submits a mainnet T2 attestation, Finalized live.

🎬 https://vimeo.com/1212224147
🔗 PR #138: https://github.com/zeroclaw-labs/zeroclaw-plugins/pull/138

`palinurus-core` v0.1.0 is on crates.io — plugins pin to a working version through any WIT ABI change. Maintained, not a one-shot. 🦞

---

## One-pager (optional Superteam attachment)

> Markdown. Attach to the Superteam submission form (optional "One-pager link" field)
> OR paste into the submission body. Distills problem → solution → proof → demo → status.

---

# Palinurus — one-pager

**Track C (DePIN & the physical edge)** · Superteam Brasil × ZeroClaw bounty

**Demo (2:03):** https://vimeo.com/1212224147
**PR #138 (submission link):** https://github.com/zeroclaw-labs/zeroclaw-plugins/pull/138
**Mainnet proof (Finalized):** https://explorer.solana.com/tx/YZTS16nNNrDhLhFHCtSMhcYTYAkcYQvPn2QUWfWvkw4bJxif77d1Ww36o3c4LYe6r69NAzYNJLDpz93DjR3G9TC

## The problem

ZeroClaw is already a DePIN device — it runs on a $40 Raspberry Pi with GPIO/I2C/SPI and an SOP engine — but it has no chain. The hard part isn't signing; it's signing *safely* with an LLM in the loop. An agent with a private key and an LLM is a hot wallet with a prompt-injection surface.

## The solution

**Palinurus** — two ZeroClaw WIT tool plugins (`wasm32-wasip2`) + a shared Solana substrate, with custody treated as a first-class engineering problem:

- **`depin-attest`** — a sensor reading → a real versioned transaction containing a Solana Attestation Service `create_attestation` instruction (memo fallback) composed with a durable nonce (the blockhash-expiry fix). **T1 default** (multisig signs) + **T2 opt-in** (scoped session key, `{System, SAS, Memo}` program allowlist, hard caps, fail-closed injection tests). **T2 path verified on mainnet.**
- **`depin-rewards`** — watches any public Helium/Hivemapper-class hotspot via the Relay API, fires real Telegram alerts on offline flips + daily rewards summaries, drafts an unsigned claim tx. **T0/T1 only — no signing key anywhere in the crate** (a test asserts it).
- **`palinurus-core`** — the shared `wasm32-wasip2` substrate (PDA derivation, base58, borsh, versioned-tx, durable-nonce, JSON-RPC over `waki`), **published on crates.io as v0.1.0**. Both plugins depend on the published crate — no fork git URL.

**Custody posture:** the agent never holds a main wallet key. Pattern: *agent proposes, multisig disposes.* T2 autonomous signing is locked to a program allowlist that **blocks value transfer** (the only allowed System ix is `AdvanceNonceAccount`), a session key holding cents, and a fail-closed prompt-injection transcript (test-backed) per plugin.

## Proven on mainnet (the moat)

The `depin-attest` T2 path is **verified on Solana mainnet** — a real, explorer-verifiable attestation paying real fees. The only on-chain-verified entry in the Track-C field, and the only one proven on mainnet (no `?cluster=devnet` in the URL — a judge verifies it directly).

| | |
|---|---|
| **Transaction** | `YZTS16nN…3G9TC` (Finalized · Mainnet Beta · slot 434472270 · 5000 lamports fee) |
| **Replay guard** | durable nonce advanced on this run — a replayed/stale attestation is rejected |
| **Custody (pre-sign)** | session-key identity · program allowlist `{System, SAS, Memo}` · lamport cap · daily cap — all enforced **before** signing |

The sensor reading committed on-chain: `palinurus: bme280-1=24.7celsius @ 1784707747` (memo). The devnet worked-example + an earlier devnet run are also explorer-verifiable — the same T2 path, two clusters, three real signed attestations.

## Demo (2:03, on camera)

https://vimeo.com/1212224147 — the real pipeline end-to-end: agent reads a public Helium hotspot → **a real Telegram alert fires the moment it goes dark** → the agent signs + submits a real T2 attestation → **explorer flips to "Finalized (MAX Confirmations)" on Mainnet Beta, live**. No mock, no slide deck, no devnet play-money.

## Build & test

```bash
rustup target add wasm32-wasip2
cargo test                                  # 212 host tests, no wasm toolchain needed
cargo build --release --target wasm32-wasip2 # core compiles to the component target
cargo clippy --all-targets -- -D warnings    # zero warnings, both crates, both feature modes
```

**212 host tests** (71 core + 83 attest + 58 rewards). `clippy -D warnings` + `wasm32-wasip2` clean.

## Status & maintenance commitment

- ✅ `palinurus-core` v0.1.0 live on crates.io.
- ✅ `depin-attest` complete (T1 + T2, 83 tests) + on-chain T2 attestation verified on mainnet.
- ✅ `depin-rewards` alerts + custody + WIT shim wired (58 tests); rewards path verified live (Relay Community tier). `claim_tx` design documented (Helium cNFT compression path — deliberately deferred, documented rather than rushed).
- **Maintenance:** `palinurus-core` is versioned on crates.io, so the plugins pin to a working version through any upstream WIT ABI change. This is a maintained submission, not a hackathon one-shot — both plugins + the core will be carried through ZeroClaw's plugin-contract evolution.

**License:** MIT. **Built by RECTOR-LABS.** 🦞

---

## Superteam Earn submission form fields (for the actual submit)

- **Demo video link (Youtube/Vimeo/GDrive)** *(required):* `https://vimeo.com/1212224147`
- **One-pager link** *(optional):* either paste the one-pager above into the submission body, or host it (e.g. push `docs/submission-drafts.md` to a gist / the repo and link the raw URL).
- **PR / repo link** *(if the form asks for code):* `https://github.com/zeroclaw-labs/zeroclaw-plugins/pull/138` (the PR) or `https://github.com/RECTOR-LABS/palinurus` (the substrate repo).

**Deadline:** submit by **Aug 7 2026**; winner announced **Aug 21 2026**.