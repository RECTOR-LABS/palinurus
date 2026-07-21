# Palinurus — demo recording guide

> The chunked shot list + per-chunk verification + voiceover script for the
> ≤3-minute Superteam demo video. Built for RECTOR's preferred workflow:
> record in chunks → verify each chunk before moving on → CIPHER generates
> the ElevenLabs voiceover → CIPHER compiles via ffmpeg into the final cut.
> **No slides.** Real ZeroClaw agent + real Telegram channel, terminal + phone.

**Deadline:** winner announced **Aug 21 2026**. The submission form *requires*
a demo video — it is the score gate. This guide exists to make recording it
fast and verifiable.

---

## 0. The story we're telling (30 seconds of context before you record)

Palinurus is **the Solana DePIN node that talks** — a $40 Raspberry Pi
running ZeroClaw becomes a Solana-attesting device, and the same agent
watches your real Helium hotspot and pings you on Telegram the moment it
goes dark. The agent **never holds a main wallet key** — it proposes, a
human or Squads multisig disposes.

The hero of the demo is a **real public Helium hotspot**: *Fit Pine
Capybara* (SenseCAP maker, owner `99Yu…oZCUm`). Over a recent window it
earned **0.02059069 HNT** (verified from a real Relay API capture, 6 reward
records, sum = 2,059,069 HNT bones) — then went **offline** (`is_active=false`).
The `depin-rewards` watch action fires a real Telegram alert the instant it
detects that online→offline flip. No hardware, no hotspot ownership, no
fake data — just a real public hotspot, a real agent, a real Telegram ping.

The demo has **two beats**:
1. **`depin-rewards` (full real, laptop only)** — the daily-use workhorse.
   The hero beat. Real Relay reads, real Telegram alert.
2. **`depin-attest` (simulated reading, Solana side 100% real)** — the
   reference implementation nobody else can build. Real unsigned versioned
   tx → real SAS `create_attestation` ix → real PDA → explorer-verifiable on
   devnet. The reading is simulated; the Solana side is not.

The wiring diagram (`docs/wiring-diagram.svg`, embedded in the README)
documents the physical-edge path the attest beat simulates. We show it
briefly on screen so judges see the Pi→Solana bridge is real, not hand-waved.

---

## 1. Pre-flight checklist (do this BEFORE pressing record)

Run every item. **Do not skip** — a mid-record failure wastes a whole chunk.

### 1a. Environment
- [ ] Terminal: a clean, dark-theme terminal (the recording background).
  Recommended: iTerm2 with a dark profile, font size ≥ 16 for legibility.
- [ ] Phone: logged into Telegram, the demo channel open, notifications ON
  with sound (the alert arriving on camera is the money shot).
- [ ] Screen recording: at 1920×1080 (or 2560×1440), 30fps, system audio OFF
  (we record voiceover separately — cleaner audio, no room noise). Use
  macOS `cmd-shift-5` → record selected area, or OBS with a display source.
- [ ] Browser: a Chrome window open to `explorer.solana.com` (devnet) ready
  to paste a PDA. Dark mode on the explorer.
- [ ] Reduce noise: close Slack/Discord/email; enable macOS Do Not Disturb.

### 1b. The repo + secrets
```bash
cd ~/local-dev/RECTOR-LABS/zeroclaw-plugins
git log --oneline -3          # confirm head = 198d2f7 (feat/depin-attest)
cd plugins/depin-rewards
cargo test                    # MUST be 49/49 green before recording
cargo clippy --all-targets -- -D warnings   # MUST be clean
```
- [ ] `cargo test` = **49 passed**.
- [ ] `cargo clippy` = **no warnings**.
- [ ] Relay key live: `source ~/Documents/secret/zeroclaw-solana/.env && \
  curl -s -H "Authorization: Bearer $RELAY_API_KEY" \
  "https://api.relaywireless.com/v1/helium/l2/hotspots/11bUUcCTeMYS4iqtf7DoQAEvmwswsZBSRc3Nt53aBQZ2ZYG8346" \
  | jq '.iot_info.is_active'` → **must print `false`** (Capybara is offline).

### 1c. The Telegram bot
- [ ] A bot exists (`@BotFather` → `/newbot`). Token in
  `~/Documents/secret/zeroclaw-solana/.env` as… actually the plugin reads
  the token + chat_id from its **config section**, not env, at runtime.
  For the demo we invoke the plugin's pure core directly (see §2), so we
  pass the token + chat_id as args to the demo driver.
- [ ] Your personal Telegram chat with the bot is open on the phone.
- [ ] Send `/start` to the bot once so it can message you.
- [ ] Confirm the chat_id: visit
  `https://api.telegram.org/bot<TOKEN>/getUpdates` → grab `chat.id`.

> **If any pre-flight item fails, STOP and fix it before recording.** A
> chunk recorded against a broken setup is wasted.

---

## 2. The demo driver (a tiny harness that calls the real plugin core)

The plugin ships as a WIT component the ZeroClaw host loads; for a
laptop demo we don't boot the full ZeroClaw runtime on camera — instead a
~40-line Rust demo binary calls the **same pure core** (`depin_rewards::`
+ `depin_attest::`) the WIT shim calls, over the real `waki` HTTP client
and the real Solana RPC. This is not a mock — it exercises the shipped
code path against live services.

> **BUILT + LIVE-VERIFIED (2026-07-21).** Two drivers (the plugins are
> standalone `[workspace]` crates, so each carries its own driver behind a
> `demo` feature, off by default):
>
> **Rewards (chunks 2-5)** — `plugins/depin-rewards`:
>   `cargo run --features demo --bin palinurus-demo -- <status|summary|watch|custody|all>`
>   `ReqwestHttp` (impl of the core `HttpClient` trait, TDD'd w/ mockito) drives
>   `do_status`/`do_summary`/`do_watch`. `watch` fires the real Telegram alert;
>   the rest are Telegram-free (safe to smoke). Live-verified: `earned 0.02 HNT`.
>
> **Attest (chunk 6)** — `plugins/depin-attest`:
>   `cargo run --features demo --bin depin-attest-demo`
>   `ReqwestRpc` (impl of `palinurus_core::Rpc`, TDD'd w/ mockito) reads the
>   live devnet durable-nonce account; `execute_t1` builds the real unsigned
>   durable-nonce tx + attestation PDA + explorer link + full base64 tx
>   (pastable into the explorer tx inspector). Config defaults to the devnet
>   artifacts provisioned 2026-07-21 (credential `feWL…` + nonce `9Kaivz…`,
>   authority = shared devnet wallet); env-overridable. No secrets (T1 unsigned).
>   Live-verified: real attestation PDA `52QV…D3uW` + real unsigned durable-nonce tx.
>
> The drivers are the only demo-only code; all logic is the shipped, tested
> plugin core (194 host tests now: 71 core + 68 attest + 49 rewards + 6 demo).

---

## 3. Shot list (7 chunks, ~3min total)

Each chunk: **record → verify (below) → if pass, move on; if fail, re-record
that chunk only.** Keep each clip as a separate file
(`chunk-01-status.mp4`, …) — ffmpeg stitches them at the end.

| # | Chunk | Duration | What's on screen | Verify |
|---|---|---|---|---|
| 1 | **Cold open + wiring** | ~20s | Terminal title card: "Palinurus — the Solana DePIN node that talks". Brief pan over the wiring-diagram SVG (open in browser, dark). | Diagram legible; no spoilers of later output. |
| 2 | **status — Capybara is offline** | ~20s | Run the driver's status step. Terminal prints: `✓ watch tall-plum-ocelot — OFFLINE`. | Output line matches the README worked example shape; `is_active=false`. |
| 3 | **summary — 0.02 HNT earned** | ~25s | Run the driver's summary step (30d window). Terminal prints the rewards line + breakdown + owner. | Total reads `0.02 HNT` (≈0.02059069); `beacon` + `witness` non-zero; `owner` short-address shown. |
| 4 | **watch → the Telegram alert (MONEY SHOT)** | ~25s | Run the driver's watch step (`prev_active=true`). Terminal prints "alert(s) sent: offline-alert". Cut to phone: the real Telegram notification drops in. | **Phone actually receives the Telegram message** with the hotspot name + "went OFFLINE". This is the beat — if the phone doesn't ping, re-record. |
| 5 | **custody one-liner** | ~10s | Terminal: `cargo test no_signing_capability_in_crate` (or show the README custody table). Voiceover: "the plugin holds no key of any kind." | Test passes / table legible. |
| 6 | **depin-attest — simulated reading → real Solana tx** | ~35s | `cd plugins/depin-attest && cargo run --features demo --bin depin-attest-demo`. Terminal prints the attestation PDA + explorer URL + the full unsigned durable-nonce tx (base64). Paste the tx into the explorer's tx inspector; paste the PDA into the address bar. | Explorer tx inspector shows the real `create_attestation` ix (program SAS, accounts, data). The PDA is recomputable from the reading. The reading is fake; the Solana artifacts are 100% real. |
| 7 | **close + CTA** | ~15s | Terminal: PR link `github.com/zeroclaw-labs/zeroclaw-plugins/pull/76` + `palinurus.rectorspace.com`. Voiceover sign-off. | Links legible; hold for 2s. |

**Total target: ~2:50.** Leaves a 10s buffer under the 3:00 hard cap.

### Timing notes
- Chunks 2–4 are the rewards beat (the 70% of screentime — it's the real,
  daily-use, no-hardware story). Don't rush them.
- Chunk 6 (attest) is the "nobody else can build this" flex — keep it tight;
  the explorer paste is the proof, not a long narration.
- If you're running long, **trim chunk 1** (the diagram pan) first — the
  diagram also lives in the README, judges can pause.

---

## 4. Per-chunk verification rubric

After recording each chunk, scrub it once and check:

1. **Legibility** — can a stranger read every line of terminal output at
   1x speed? (If not: increase font size, re-record.)
2. **No secrets on screen** — the Relay key, Telegram bot token, and any
   RPC API key must NEVER appear. The driver loads them from env/config off
   camera; verify no `echo $RELAY_API_KEY` sneaks into a chunk. If a secret
   is visible, **delete that clip immediately** and re-record.
3. **The beat landed** — did the specific "verify" column item happen?
4. **Clean frame** — no notifications, no unrelated windows, no terminal
   scrollback of previous commands bleeding in. Use `clear` between chunks.

> **The money shot (chunk 4) is non-negotiable.** If the phone does not
> receive the live Telegram alert on camera, the demo loses its punch.
> Re-record until it lands. Tip: send a manual test message from the bot
> right before the take so you know notifications are firing, then delete
> that test message so the on-camera alert is the first one.

---

## 5. Voiceover script (ElevenLabs)

Tone: calm, technical, confident — a senior engineer showing real work, not
a hype reel. ~150 words → ~55s of speech, leaving ~2:05 of "let the screen
breathe" time. Record the VO to match the chunks; ffmpeg aligns timing.

> **Voice / model:** ElevenLabs — pick a warm, mid-range male voice (e.g.
> "Adam" or "Antoni" preset), stability ~55, similarity ~75, style ~20.
> Generate per-chunk so a bad take doesn't poison the whole VO.

---

**[Chunk 1 — cold open + wiring]**
> Palinurus is the Solana DePIN node that talks. A forty-dollar Raspberry
> Pi running ZeroClaw becomes a Solana-attesting device — and the same
> agent watches your real Helium hotspot and pings you the moment it goes
> dark. The agent never holds a main wallet key.

**[Chunk 2 — status]**
> Here's a real public Helium hotspot — Fit Pine Capybara. The plugin
> reads its status from the Relay API. It's offline.

**[Chunk 3 — summary]**
> Over the last thirty days it earned about point-zero-two HNT — beacon
> and witness rewards, summed client-side from the public reward shares.
> No hotspot ownership required. This is data anyone can read.

**[Chunk 4 — watch → Telegram alert]**
> The watch action detects the online-to-offline flip and fires a real
> Telegram alert to the owner. That's a real message on a real phone.

**[Chunk 5 — custody]**
> And here's the custody posture: the plugin holds no key of any kind.
> No signing dependency, no signing code path anywhere in the crate.
> The agent proposes; a human or a Squads multisig disposes.

**[Chunk 6 — depin-attest]**
> The second plugin, depin-attest, takes a sensor reading — simulated
> here — and builds a real unsigned Solana transaction: a Solana
> Attestation Service create-attestation instruction, composed with a
> durable nonce so it survives an approval queue. The reading is fake.
> The attestation PDA, the instruction, the explorer link — those are real.

**[Chunk 7 — close]**
> Two plugins, one minimal substrate on crates.io, a hundred and
> eighty-eight tests, all wasm-clean. Track C — the DePIN lane nobody
> else entered. PR link and the project site are on screen.

---

## 6. Compile (ffmpeg) — CIPHER does this

After all 7 chunks are verified + the VO is generated:

```bash
# 1. Concatenate the screen chunks (re-encode to a common format first)
for f in chunk-0*.mp4; do
  ffmpeg -y -i "$f" -c:v libx264 -preset fast -crf 20 -an "prep-$(basename $f)"
done
printf "file 'prep-chunk-01.mp4'\nfile 'prep-chunk-02.mp4\n…" > concat.txt
ffmpeg -y -f concat -safe 0 -i concat.txt -c copy screen.mp4

# 2. Mix the per-chunk voiceover into one track aligned to chunk timings
#    (voiceover chunks generated separately; concat silence to align)
ffmpeg -y -i vo-01.mp3 -i vo-02.mp3 … -filter_complex \
  "[0:a]adelay=0|0[a0]; [1:a]adelay=20000|20000[a1]; … ; \
   [a0][a1]…amix=inputs=7" vo.mp3

# 3. Mux screen + voiceover, cap at 3:00
ffmpeg -y -i screen.mp4 -i vo.mp3 -c:v copy -c:a aac -shortest \
  -t 180 palinurus-demo.mp4
```

**Final QC before upload:**
- [ ] Total duration ≤ 3:00 (aim 2:45–2:55).
- [ ] Audio level peaks around −6 dB, no clipping.
- [ ] No secret visible in any frame (scrub at 0.25x once more).
- [ ] Phone alert (chunk 4) is clearly visible + audible-ish (even though VO
      is separate, the notification should align with the VO "real message
      on a real phone" line).
- [ ] Links in chunk 7 are legible at 1x and held ≥ 2s.

Upload to YouTube (unlisted) or a Superteam-accepted host, grab the link,
use it as the demo video URL on the Superteam Earn submission form.

---

## 7. What "done" looks like for the demo track

- [x] Wiring diagram SVG (`docs/wiring-diagram.svg`) — done.
- [ ] This recording guide — done when committed.
- [x] Demo drivers — DONE + live-verified (palinurus-demo rewards + depin-attest-demo attest, PR #76).
- [ ] Marketing site (`palinurus.rectorspace.com`) — scaffolds next, embeds
      this video.
- [ ] All 7 chunks recorded + verified.
- [ ] Voiceover generated.
- [ ] Final `palinurus-demo.mp4` ≤ 3:00, QC passed.
- [ ] Submitted on Superteam Earn (PR #76 = submission link; video = required;
      optional: tweet + one-pager).
- [ ] Posted in `#solana-bounty` on the ZeroClaw Discord with the PR + video.

---

## 8. Risk register (things that could bite the record)

| Risk | Mitigation |
|---|---|
| Relay API 5xx / rate limit mid-take | Pre-flight curl (§1b) confirms live; keep takes short; the driver retries once. |
| Telegram notification doesn't arrive on camera | Send a manual test message first to confirm notifications, delete it, then roll. Keep the phone unlocked + screen on. |
| A secret leaks into a frame | Driver loads secrets from env off-camera; `clear` between chunks; scrub at 0.25x in QC. |
| Font too small for judges | font-size ≥ 16; verify legibility rubric per chunk. |
| Goes over 3:00 | Trim chunk 1 first; the diagram is in the README too. |
| `cargo test` not green | Pre-flight blocks recording — fix before, never during. |
| Capybara flips back online before recording | Re-capture: the story still holds (online→offline flip), just pick a fresh offline hotspot or re-frame. The driver's `prev_active` arg makes the flip deterministic regardless of live state. |