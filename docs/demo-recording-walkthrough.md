# Palinurus — chunk-by-chunk recording walkthrough

> The actionable companion to `demo-recording-guide.md`. You record one chunk
> at a time, hand me the file, I verify against the checklist, and only when
> I say **PASS** do you record the next chunk. This doc is the per-chunk
> script + my verification gate. The recording guide holds the story (§0),
> pre-flight (§1), voiceover script (§5), and ffmpeg compile (§6) — read it
> once before you start.

## Workflow

```
for each chunk 1..7:
  1. you: clear the terminal, run the chunk's command(s), record the clip
  2. you: save it as docs/recording/chunk-NN-<name>.mp4
  3. you: tell me the file path
  4. me: I verify against the checklist (legibility / no secrets / the beat landed / clean frame / chunk-specific)
  5. me: PASS → record the next chunk   |   FAIL (reason) → re-record this chunk only
```

**Hard rules**
- **Never record the next chunk before I PASS the current one.** A bad chunk discovered late wastes everything after it.
- **No secret on screen, ever.** The Relay key, the Telegram bot token, the Solana keypair — none may appear. The drivers load them from env off-camera. If a secret leaks into a frame, **delete that clip immediately** and re-record. I scrub for this every chunk.
- **One clip per chunk** (`chunk-01-status.mp4`, `chunk-02-summary.mp4`, …) in `docs/recording/`. ffmpeg stitches them at the end.
- **System audio OFF.** We record voiceover separately (cleaner). The phone notification sound in chunk 4 is fine to capture but not required.

---

## Pre-flight (do once, before chunk 1)

From `demo-recording-guide.md` §1 — run every item. Highlights:

```bash
# repo head + tests green (counts updated to the C1 commit)
cd ~/local-dev/RECTOR-LABS/zeroclaw-plugins
git log --oneline -1            # expect: c81ccdd  feat(depin-attest): T2 demo driver…
cd plugins/depin-rewards
cargo test --features demo      # MUST be 52 passed (49 core + 3 demo)
cargo clippy --features demo --all-targets -- -D warnings   # MUST be clean
cd ../depin-attest
cargo test --features demo      # MUST be 74 passed (68 core + 6 demo)
cargo clippy --features demo --all-targets -- -D warnings   # MUST be clean

# Relay live + Capybara offline (the hero)
source ~/Documents/secret/zeroclaw-solana/.env
curl -s -H "Authorization: Bearer $RELAY_API_KEY" \
  "https://api.relaywireless.com/v1/helium/l2/hotspots/11bUUcCTeMYS4iqtf7DoQAEvmwswsZBSRc3Nt53aBQZ2ZYG8346" \
  | jq '.iot_info.is_active'    # MUST print false
```
- [ ] Terminal: dark theme, font ≥ 16, 1920×1080 (or 1440p), 30fps, sys audio OFF.
- [ ] Phone: Telegram open to the bot chat, notifications ON + sound, screen unlocked.
- [ ] Chrome: dark mode, `explorer.solana.com` open.
- [ ] macOS Do Not Disturb ON; Slack/Discord/email closed.
- [ ] Telegram bot sends you a test message (then delete it) to confirm notifications fire.

> Any pre-flight fail → STOP, fix, then start recording. Don't record against a broken setup.

---

## Chunk 1 — cold open + wiring (~20s)

**Run / capture**
```bash
clear
# open docs/wiring-diagram.svg in Chrome (dark), or show the README hero on GitHub
# title card text (in terminal or a slide overlay): "Palinurus — the Solana DePIN node that talks"
```
- Pan slowly over the wiring diagram (Physical Edge → Agent → Plugins → Solana) so each column is legible.
- Hold the title card ~3s.

**Send me:** `docs/recording/chunk-01-wiring.mp4`

**I verify**
- [ ] Diagram columns legible at 1x (Pi, ZeroClaw, depin-attest/depin-rewards, Solana).
- [ ] No later-chunk output spoiled (no terminal scrollback of the rewards/attest runs).
- [ ] Title text readable, held ≥ 2s.
- [ ] No notifications / no unrelated windows in frame.

---

## Chunk 2 — status: Capybara is offline (~20s)

**Run / capture**
```bash
clear
cd ~/local-dev/RECTOR-LABS/zeroclaw-plugins/plugins/depin-rewards
source ~/Documents/secret/zeroclaw-solana/.env   # off-camera; do NOT show the key
cargo run --features demo --bin palinurus-demo -- status
```
Expected output shape: `✓ Fit Pine Capybara — OFFLINE (iot) · owner 99Yu…ZCUm · maker SenseCAP` (or the custody-prefixed line the driver prints).

**Send me:** `docs/recording/chunk-02-status.mp4`

**I verify**
- [ ] The hotspot name **Fit Pine Capybara** is visible + the **OFFLINE** state is visible.
- [ ] Output matches the README worked-example shape.
- [ ] **No secret on screen** — the `source .env` line must NOT be in the frame (run it off-camera, or `clear` after sourcing before the `cargo run`).
- [ ] Font legible at 1x.

---

## Chunk 3 — summary: 0.02 HNT earned (~25s)

**Run / capture**
```bash
clear
cargo run --features demo --bin palinurus-demo -- summary
```
Expected: total reads `0.02 HNT` (≈0.02059069), `beacon` + `witness` non-zero, owner short-address.

**Send me:** `docs/recording/chunk-03-summary.mp4`

**I verify**
- [ ] Total reads **0.02 HNT** (the hero number — must be visible).
- [ ] `beacon` + `witness` breakdown shown and non-zero.
- [ ] Owner short-address visible.
- [ ] No secret; clean frame; legible.

---

## Chunk 4 — watch → the Telegram alert (MONEY SHOT, ~25s)

**Run / capture**
```bash
clear
# watch fires the real alert. prev_active=true forces the offline-flip detection.
cargo run --features demo --bin palinurus-demo -- watch
```
- The terminal prints `alert(s) sent: offline-alert`.
- **Cut to the phone** (keep recording — one continuous clip): the real Telegram notification drops in with the hotspot name + "went OFFLINE".

**Send me:** `docs/recording/chunk-04-watch.mp4`

**I verify (this is non-negotiable)**
- [ ] **The phone actually receives the Telegram message** with the hotspot name + offline alert. If it doesn't ping, FAIL → re-record.
- [ ] Terminal shows `alert(s) sent: offline-alert`.
- [ ] The notification sound fires (nice-to-have; sys audio can be off).
- [ ] No secret; the bot token is NOT visible anywhere.
- [ ] Tip if it won't land: send a manual test message first to confirm notifications, delete it, then roll.

---

## Chunk 5 — custody one-liner (~10s)

**Run / capture**
```bash
clear
# Option A (test-backed): run the no-signing invariant test
cargo test --features demo no_signing_capability_in_crate -- --nocapture
# Option B (cleaner on screen): show the README custody table (scroll to it in the browser)
```

**Send me:** `docs/recording/chunk-05-custody.mp4`

**I verify**
- [ ] The "no signing key anywhere in the crate" assertion is visible (test passes, or the table is legible).
- [ ] The custody tiers (T0/T1/T2) are readable.
- [ ] No secret; clean frame.

---

## Chunk 6 — depin-attest: real on-chain attestation (the killer beat, ~35s)

> **Updated from the original guide.** This is now the **T2 path** — the agent
> SIGNS + SUBMITS a real attestation to devnet. Far more compelling than the
> T1 unsigned draft: you land a real confirmed tx on camera and open it in
> the explorer. This is the beat none of the 3 competitors can show.

**Run / capture**
```bash
clear
cd ~/local-dev/RECTOR-LABS/zeroclaw-plugins/plugins/depin-attest
# off-camera: the keypair path is a secret-adjacent path; don't show the file contents.
ATTEST_MODE=t2 ATTEST_MEMO_FALLBACK=true \
ATTEST_SESSION_KEYFILE=~/Documents/secret/solana-devnet.json \
cargo run --features demo --bin depin-attest-demo
```
- Terminal prints: `✓ attested + submitted`, the **signature**, the explorer link, and `✅ real on-chain attestation`.
- **Wait ~6 seconds** (let the tx finalize — it's not instant), then **open the explorer link** in Chrome. The page shows **Success / Finalized**.
- Scroll so the signature + Status + the Memo + System program accounts are visible.

**Send me:** `docs/recording/chunk-06-attest.mp4`

**I verify**
- [ ] The command + `✓ attested + submitted` + signature + `✅ real on-chain attestation` line are visible.
- [ ] **The explorer page shows Success / Finalized** for the same signature the terminal printed (they must match — if the explorer shows "Not found", you didn't wait long enough; re-record and wait ~8s before opening).
- [ ] The Memo Program + System Program accounts are visible (proves it's a memo + durable-nonce tx).
- [ ] No secret: the keypair **file contents** must never appear (the command shows the *path*, which is fine; do not `cat` the keypair).
- [ ] The reading is simulated (bme280-1=24.7) — that's expected + honest; the Solana side is 100% real.
- [ ] Font legible; explorer in dark mode.

> **Note on multiple takes:** each T2 run is a REAL new devnet tx (advances the nonce, costs ~0.000005 SOL). That's fine — devnet, the wallet has 5.24 SOL. Don't be shy about re-rolling for a clean take.

---

## Chunk 7 — close + CTA (~15s)

**Run / capture**
```bash
clear
# show in terminal or browser:
#   github.com/zeroclaw-labs/zeroclaw-plugins/pull/76
#   palinurus.rectorspace.com
# (optionally the README hero on GitHub)
```
- Hold the PR link + site link legibly for ≥ 2s each.

**Send me:** `docs/recording/chunk-07-close.mp4`

**I verify**
- [ ] Both links legible at 1x, held ≥ 2s.
- [ ] No secret; clean frame.

---

## After all 7 chunks PASS

1. **I generate the ElevenLabs voiceover per-chunk** using `demo-recording-guide.md` §5, timed to each chunk's real duration (I'll measure each clip + write the VO to match — the script in §5 is the draft; I refine pacing to fit). Voice: warm mid-range male (Adam/Antoni), stability ~55, similarity ~75, style ~20.
2. **I compile via ffmpeg** (guide §6): concat the 7 chunks + mux the per-chunk VO → `palinurus-demo.mp4` ≤ 3:00.
3. **Final QC** (I do this, then you sign off): ≤ 3:00 · audio peaks ~−6 dB no clipping · no secret in any frame (scrub at 0.25x) · chunk 4 phone alert aligns with the VO "real message on a real phone" line · chunk 7 links held ≥ 2s.
4. **Upload** (YouTube unlisted or Superteam-accepted host) → grab the URL.
5. **Post F1 (Discord) + F2 (X)** with the PR + devnet proof + **the demo video embedded**.
6. **Submit on Superteam Earn** (PR #76 = submission link, video URL = required, optional one-pager).
7. **Embed the video on the marketing site** + redeploy.

---

## Quick reference — what I check on every chunk (the 4 gates)

1. **Legibility** — every terminal line readable at 1x. (Else: bigger font, re-record.)
2. **No secrets** — Relay key, Telegram token, Solana keypair contents never on screen. (Else: delete clip, re-record.)
3. **The beat landed** — the chunk-specific "I verify" checklist item happened.
4. **Clean frame** — `clear` between chunks, no notifications, no scrollback bleed.

Chunk 4 (Telegram alert) + chunk 6 (explorer Confirmed) are the two non-negotiable beats. Everything else is polish.