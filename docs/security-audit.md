# Palinurus `depin-attest` T2 — Security Audit (Phase A read-only → Phase B hardened)

> Auditor-grade review of the **T2 autonomous-signing path** of `depin-attest`.
> Methodology adapted from `solana-cpi-safety-skill/agents/cpi-auditor.md`:
> **read-only, cite `file:line` for every claim, skeptical posture (clean grep ≠
> pass), checklist-driven, structured findings, evidence over assertion.**
>
> **Scope:** `plugins/depin-attest/src/{lib.rs, depin_attest.rs}` + the
> `palinurus-core` crypto-critical primitives in the T2 path (`versioned_tx.rs`,
> `durable_nonce.rs`, `base58.rs`, `rpc.rs`). `depin-rewards` is out of primary
> scope (T0/T1, no signing).
>
> **Caveat (honest):** this is a guidance-driven review, not a guaranteed-
> complete scanner. "Review boundary" marks a property that depends on code
> outside the audited set (host contract / on-chain behavior) and must be
> confirmed, not assumed clean.
>
> **Date:** 2026-07-21. **Auditor:** CIPHER. **Status:** Phase B COMPLETE — all of F1–F7 followed up; mainnet proof + public Security section next.

---

## Findings summary

| # | Pattern | Location (`file:line`) | Severity | Status |
|---|---|---|---|---|
| F1 | T2 custody trusts `__config` in args is host-injected & not LLM-overridable | `lib.rs:103-110`; **host: `zeroclaw/crates/zeroclaw-plugins/src/runtime.rs:157-168`** | **Resolved — safe by host contract** | ✅ Closed — host strips caller `__config` + re-injects trusted config; host-tested |
| F2 | T2 program allowlist is **program-level**; `System` is allowed → a `System::Transfer` ix passes the guard | `depin_attest.rs:723` (`T2_ALLOWED_PROGRAMS`), `:733-744` (`enforce_program_allowlist`) | **Medium-High** | ✅ Closed — instruction-level System allowlist (disc `0x04` only); +3 TDD tests |
| F3 | Per-day cap is **not a security control**: `today_utc` derived from the LLM-supplied `timestamp` (attacker-rolls it) + `thread_local` (resets on reload) | `depin_attest.rs:786` (`enforce_daily_cap`), `:~905` (`today_utc = reading.timestamp / 86400`), `:792-802` (`DailyCapState`) | **Low-Medium** *(disclosed; bounded by session-key balance)* | ✅ Followed-up — disclosure sharpened (rate-hint, not guard); on-chain counter PDA roadmaped |
| F4 | Session key not zeroized; raw key bytes linger on the stack in `from_section` | `depin_attest.rs:~250-270` (session-key parse), `AttestConfig.session_key` | **Low** *(defense-in-depth; not persisted across calls; wasm-sandboxed)* | ✅ Closed — `zeroize` the raw session-key bytes after `SigningKey::from_bytes` |
| F5 | Attacker-supplied `memo` length is unbounded → oversized tx / RPC waste | `depin_attest.rs:~290` (`build_memo_ix`), execute paths | **Low** *(no fund risk; output shaping caps the preview)* | ✅ Closed — `MEMO_MAX_BYTES=566` + `validate_memo` wired into all 3 execute paths; +1 test |
| F6 | Nonce account **owner** not verified to be the System program | `depin_attest.rs` execute_t1/t2 (after `parse_nonce_account`) | **Low** *(defense-in-depth; `nonce_account` is config-trusted)* | ✅ Closed — `acct.owner == System` check before `parse_nonce_account` (all 3 paths); +1 test |
| F7 | `expect()`/panic sites in crypto-critical paths (panic in a wasm component = abort, not a controlled error) | `depin_attest.rs:~216` (`encode`), `versioned_tx.rs:~210,225,232` (`build_unsigned`) | **Low** *(unreachable inputs; robustness)* | ✅ Accepted w/ rationale — unreachable panic documented in-code; core `Result` variant roadmaped for a future `palinurus-core` bump |
| P1 | Signing correctness: `execute_t2` signs `serialize_message` (raw V0 bytes) — matches Solana's raw-bytes ed25519 verify; **empirically confirmed** by the devnet landing (the demo driver calls the same `execute_t2`) | `depin_attest.rs:~915` (`sign(&msg_bytes)`), `versioned_tx.rs:~243` (`serialize_message`) | — | **Passing** — recommend a `solana_sdk` cross-impl oracle for byte-equivalence rigor |
| P2 | Identity guard enforces `session_key.vk == authority == payer == nonce_authority` **before** any signing | `depin_attest.rs:748-784` (`enforce_session_key_identity`) | — | **Passing** |
| P3 | Durable-nonce replay guard live: nonce advances on every submit; a replayed/stale signed tx is rejected on-chain | `durable_nonce.rs:91` (`ADVANCE_NONCE_ACCOUNT_IX_DATA=[0x04,0,0,0]`), execute_t2 submits via `send_transaction` | — | **Passing** (devnet-confirmed: nonce advanced `F3tGxZwV… → HxmL2Nu7…`) |
| P4 | `value`/`timestamp`/`sensor_id`/`unit` input validation is fail-closed | `depin_attest.rs` `validate_reading` (finite value, positive timestamp, non-empty id/unit) | — | **Passing** |
| P5 | `Debug` for `AttestConfig` redacts `session_key` + `rpc_api_key` | `depin_attest.rs:131-170` (manual `Debug`) | — | **Passing** |
| P6 | Signed tx (`tx_b64`) is computed but **not surfaced** to the LLM (`ToolResult.output = summary` only; `shape_t2` omits tx bytes) | `lib.rs` (returns `out.summary`), `depin_attest.rs:~430` (`shape_t2`) | — | **Passing** (and replay-protected by nonce advance even if leaked) |

---

## Per-finding detail

### F1 — T2 custody trusts `__config` in args is host-injected (CRITICAL — review boundary)

**What.** `lib.rs:103-110` deserializes a `__config: HashMap<String,String>` field out of the `execute(args)` JSON, and `lib.rs:~155` builds the entire `AttestConfig` (including `session_key`, `authority`, `nonce_authority`, `rpc_endpoint`) from it via `from_section`. The LLM-facing `parameters_schema` (`lib.rs:~87`) advertises only `sensor_id/value/unit/timestamp/memo` — **not** `__config`.

**Why exploitable (if the assumption is wrong).** The whole custody model is only as trustworthy as the source of `__config`. If the ZeroClaw host **merge-trusts** an LLM-supplied `__config` (rather than authoritatively overwriting it from the operator's config), a prompt-injected agent could supply its own `__config` and override `authority` / `nonce_authority` / `session_key` / `rpc_endpoint`. Defense-in-depth *limits* the blast radius (the identity guard F P2 rejects a session key that doesn't equal authority/payer/nonce_authority; the allowlist F2 blocks transfers; the main wallet key is never in config), so the realistic worst case is **DoS / integrity-spoofing / rpc-endpoint redirect (SSRF)**, not main-wallet theft — but a foundational trust assumption cannot rest on "probably."

**Why it's likely safe-by-convention.** The canonical reference plugin `redact-text` uses the *identical* pattern — `redact-text/src/lib.rs:44` `#[serde(rename = "__config", default)]`, `redact-text/src/lib.rs:102` `RedactConfig::from_section(&parsed.config)`, documented as "the plugin's own jailed config section (`config_read` permission)." The manifest-declared `config_read` permission is the host's grant to inject config. So the host convention is clearly "inject trusted operator config into args as `__config`."

**Resolution — confirmed safe by host contract.** The ZeroClaw host (`zeroclaw-labs/zeroclaw/crates/zeroclaw-plugins/src/runtime.rs:157-168`) does exactly the safe thing: it **`obj.remove("__config")`** (strips any caller/LLM-supplied `__config` — anti-spoofing) **then re-injects the plugin's resolved operator config under `__config`**. The host also withholds the section entirely when the manifest lacks `config_read` (test `effective_config_withholds_section_without_config_read`, `runtime.rs:239`) and the anti-spoof path is tested (`inject_config_strips_caller_supplied_config_when_section_…`, `runtime.rs:230-235`: a forged `{"__config":{"api_key":"forged"}}` is replaced with `"real"`). So the trust assumption holds — and is **explicitly defended + tested** by the host. No plugin change needed; the finding is closed with a host-source citation.

---

### F2 — T2 program allowlist is program-level; `System::Transfer` passes the guard (MEDIUM-HIGH)

**What.** `depin_attest.rs:723` `T2_ALLOWED_PROGRAMS = [System, SAS, Memo]`. `depin_attest.rs:733-744` `enforce_program_allowlist` checks **only `ix.program_id`**. `System` is in the allowlist, and `SystemInstruction::Transfer` (move SOL) is a valid System instruction → **a `System::Transfer` ix would pass the guard.**

**Why exploitable (today: not directly).** The plugin only ever *constructs* `AdvanceNonceAccount` (`durable_nonce.rs:91` discriminator `[0x04,0,0,0]`, auto-prepended by `build_with_durable_nonce`) — it never constructs a `Transfer`. So it is safe **by construction**, and the blast radius of any hypothetical injected `Transfer` is bounded by the session-key balance (cents; the session key is `payer`). **But** the README/PR claim *"a value-transfer instruction is not expressible"* is true only at the construction layer, **not** at the guard layer. A reviewer reading `enforce_program_allowlist` sees program-level only.

**Fix sketch (the headline hardening).** Tighten the allowlist to **instruction-level for `System`**: any `System`-program ix must have `data[0..4] == [0x04,0,0,0]` (`AdvanceNonceAccount`). That makes value-transfer *truly unexpressible at the guard*, matching the claim. ~15 lines + 2 tests (reject `Transfer` disc `0x02`; accept `AdvanceNonceAccount` disc `0x04`). `SAS`/`Memo` can stay program-level (neither moves SOL; SAS creates an attestation account funded by the payer within the lamport cap).

---

### F3 — Per-day cap is not a security control (LOW-MEDIUM, disclosed)

**What.** `depin_attest.rs:~905` `let today_utc = reading.timestamp / 86400;` — the "day" is derived from the **reading's timestamp**, which is an LLM-supplied arg (attacker-influenced under prompt injection). `enforce_daily_cap` (`:786`) resets the counter when `today_utc` advances. `DailyCapState` (`:792-802`) is `thread_local` → resets on component reload.

**Why exploitable.** An attacker who can call `execute` rolls the `timestamp` to advance `today_utc`, resetting the cap arbitrarily; component reload also resets it. So the cap is a **rate-hint, not a guard**. The real fund bound is the **session-key lamport balance** (the `max_lamports_per_tx` cap bounds each tx; the key runs dry). The README discloses the soft/reload nature and the timestamp-rolling, but **under-frames** that this makes the cap a non-guard — and the **memo T2 path has no PDA dedup** (unlike SAS, where identical readings collide on the attestation PDA), so a timestamp-rolling flood on the memo path drains the key faster.

**Fix sketch.** (a) Re-document honestly: "the daily cap is a rate-hint, not a hard guard; the hard bound is the session-key balance + per-tx lamport cap." (b) The real fix is an on-chain counter PDA incremented per attestation (roadmap — needs a program; defeats the "no custom program" simplicity). Not a pre-deadline code fix; it's a documentation-honesty fix now.

---

### F4 — Session key not zeroized (LOW)

**What.** `from_section` (`depin_attest.rs:~250-270`) base58-decodes the session key into a stack `arr: [u8;32]`, copies into `SigningKey::from_bytes`. Neither `arr` nor the `SigningKey` is zeroized; raw bytes linger on the stack / in the `AttestConfig` until dropped.

**Why exploitable.** Low. In the shim, `AttestConfig` is rebuilt per-`execute` call (not persisted), so the key lives only during one call. The wasm component's linear memory is sandboxed from the host. But for a money-critical path, defense-in-depth says zeroize.

**Fix sketch.** Bring in `zeroize` (already a transitive ed25519-dalek dep), derive/implement `ZeroizeOnDrop` for the key-holding path, or `arr.zeroize()` after `from_bytes`. Optional.

---

### F5 — Memo length unbounded (LOW)

**What.** `build_memo_ix` (`depin_attest.rs:~290`) does `memo.as_bytes().to_vec()` with no length cap. The attacker-supplied `memo` (from args) can be arbitrarily large.

**Why exploitable.** No fund risk. A huge memo → tx exceeds Solana's 1232-byte limit → `sendTransaction` rejects (fail-closed-ish; wastes an RPC call + a nonce-account read). Output shaping caps the memo *preview* to 60 chars (`shape_*`), so no context-flood via output.

**Fix sketch.** Cap `memo.len()` (e.g., ≤566 bytes, the Memo program's hard limit, or a tighter ≤200). Reject longer → `AttestError::InvalidReading`.

---

### F6 — Nonce account owner not verified (LOW)

**What.** execute_t1/t2 call `parse_nonce_account(&acct.data)` and check `nonce_data.authority == cfg.nonce_authority`, but never check `acct.owner == Pubkey::SYSTEM`. `parse_account_info` (`rpc.rs`) returns `owner`.

**Why exploitable.** Low. `cfg.nonce_account` is config-trusted (assuming F1 holds), so an attacker can't redirect it. And a non-System account fed to `AdvanceNonceAccount` fails on-chain. But verifying the owner is cheap defense-in-depth.

**Fix sketch.** After `get_account_info`: `if acct.owner != Pubkey::SYSTEM { return Err(NonceAccount("nonce account owner is not System")); }`.

---

### F7 — `expect()`/panic sites in crypto-critical paths (LOW)

**What.** `SensorReading::encode` (`depin_attest.rs:~216`) `borsh::to_vec(self).expect("infallible")`; `build_unsigned` (`versioned_tx.rs:~210,225,232`) `u8::try_from(...).expect("<=255")` + `index_map.get(...).expect(...)`.

**Why exploitable.** None of these are reachable for our inputs (fixed-shape struct; ≤~10 accounts). But in a wasm component, a panic = an abort the host sees as an error (not a controlled `Result`). For a money-critical path, returning `Err` is more defensible than panicking.

**Fix sketch.** Convert the `expect`s to `?` with a typed error (the `>255 accounts` case → `AttestError::Borsh`/a new variant; unreachable but typed).

---

## Coverage summary (12 lenses)

| # | Lens | Result |
|---|---|---|
| 1 | Custody-guard completeness | **F2** (System instruction-level gap); identity + caps enforced at every T2 entry (P2) |
| 2 | Crypto correctness | **P1** signing-correct (devnet-confirmed; recommend solana_sdk oracle); PDA/nonce derivation oracle-verified in `palinurus-core` |
| 3 | Tx construction | **P3** durable-nonce correct (`durable_nonce.rs` oracle-verified); **F7** panic-vs-Result |
| 4 | Secret handling | **F4** no zeroize; **P5** Debug redacts key; key not persisted across calls |
| 5 | Input validation | **P4** reading validated; **F5** memo unbounded |
| 6 | Replay / front-running | **P3** nonce-advance replay guard (devnet-confirmed); attestation-PDA dedup on SAS path |
| 7 | Error handling / atomicity | guards enforced **before** signing (`execute_t2` order); **F7** panic robustness |
| 8 | Output shaping | **P6** signed tx not surfaced; ≤200-token summaries; no raw dumps |
| 9 | Integer / overflow | TTL `checked_add`; `enforce_lamport_cap` `saturating_mul`; `overflow-checks=true` now on; no logic bug found |
| 10 | Dependency / sandbox | `palinurus-core = "0.1"` pinned; `waki`/`wasip2` http only; perms minimal (`http_client`+`config_read`) |
| 11 | Edge / logic | **F3** cap-rollover; **F6** nonce owner; UTC-midnight rollover handled by `enforce_daily_cap` |
| 12 | WIT trust boundary | **F1 — RESOLVED**: host (`runtime.rs:157-168`) strips caller `__config` + injects trusted config (host-tested) — `__config` cannot be spoofed |

---

## Phase B hardening — COMPLETE (each fix = TDD evidence: failing test → green)

| # | Finding | Disposition | Evidence |
|---|---|---|---|
| F2 | Instruction-level System allowlist | ✅ Fixed | `enforce_program_allowlist` rejects any System ix ≠ `AdvanceNonceAccount` (disc `0x04`); +3 tests (`prompt_injection_system_transfer_rejected`, `_short_system_ix_rejected`, `_system_advance_nonce_allowed`) |
| F1 | `__config` trust boundary | ✅ Resolved (host) | Confirmed: `zeroclaw/crates/zeroclaw-plugins/src/runtime.rs:157-168` strips caller `__config` + re-injects trusted config (host-tested) |
| F5 | Memo length cap | ✅ Fixed | `MEMO_MAX_BYTES=566` + `validate_memo` in t1/memo_fallback/t2; `execute_t1_rejects_oversized_memo` |
| F6 | Nonce account owner check | ✅ Fixed | `acct.owner == System` before parse (all 3 paths); `execute_t1_rejects_non_system_nonce_owner` |
| F4 | Zeroize session key | ✅ Fixed | `zeroize` on `arr` + `key_bytes` after `SigningKey::from_bytes` (defense-in-depth) |
| F3 | Daily-cap honesty | ✅ Followed-up | README disclosure sharpened: rate-hint, not guard; real bounds = allowlist + session-key balance; on-chain counter PDA roadmaped |
| F7 | Unreachable `expect()` panics | ✅ Accepted w/ rationale | `encode()` `expect` documented in-code (infallible: fixed-shape struct, <100B Vec); `build_unsigned` panics in published core — `Result` variant roadmaped |

**Result:** attest 78 → 83 host tests (+5: F2×3, F5×1, F6×1). clippy `-D warnings` + `wasm32-wasip2` release clean. Repo validation exit 0. **No critical vulnerability remains** — the T2 path is hardened; the one foundational trust assumption (`__config`) is confirmed safe by the host contract.

**Next:** mainnet T2 proof (deploy the *hardened* version) → fold this report into a public "Security" section in the README/PR (the closed audit loop = the evidence).
---

*Wallahu a'lam. This review is guidance-driven and cite-backed; it is not a guarantee of completeness. Every "Passing" note above is backed by the cited code location + (where applicable) the devnet landing.*
