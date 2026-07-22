import { Metadata } from "next";

export const metadata: Metadata = {
  alternates: { canonical: "/" },
};

const PR_URL = "https://github.com/zeroclaw-labs/zeroclaw-plugins/pull/76";
const REPO_URL = "https://github.com/RECTOR-LABS/palinurus";
const BOUNTY_URL = "https://superteam.fun/earn/listing/zeroclaw";
const CORE_CRATES = "https://crates.io/crates/palinurus-core";

const facts = [
  { k: "212", v: "host tests, all green" },
  { k: "wasm32-wasip2", v: "clean component build" },
  { k: "v0.1.0", v: "palinurus-core on crates.io" },
  { k: "mainnet", v: "real on-chain T2 attestation — verified" },
];

const plugins = [
  {
    name: "depin-attest",
    accent: "attest" as const,
    tag: "the reference implementation nobody else can build",
    body: "A sensor reading flows in → the plugin builds a real versioned transaction containing a Solana Attestation Service create_attestation instruction, composed with a durable nonce (so it survives an approval queue). The reading is simulated in the demo; the PDA, the instruction, the signature, and the explorer link are real. The T2 custody path (scoped session key, {System, SAS, Memo} allowlist, caps) is verified live on mainnet — a real confirmed attestation paying real fees (sig YZTS16nN…3G9TC, Finalized, Mainnet Beta).",
    points: ["83 host tests", "T2 verified on mainnet", "memo fallback + SAS path"],
    custody: "T1 default · T2 opt-in (verified on mainnet)",
  },
  {
    name: "depin-rewards",
    accent: "rewards" as const,
    tag: "the daily-use workhorse — no hardware, no hotspot ownership",
    body: "Watch any public Helium hotspot via the Relay API. Get a Telegram ping the moment it goes offline, and a daily rewards summary at 08:00. Verified live against a real hotspot (Fit Pine Capybara — earned 0.02 HNT, then went dark). No signing key anywhere in the crate.",
    points: ["58 host tests", "real Relay + Telegram", "T0 reads · T1 unsigned · no T2"],
    custody: "T0 reads · T1 unsigned · no T2",
  },
];

const tiers = [
  { tier: "T0", color: "attest", desc: "Read public data. No key, no risk." },
  { tier: "T1", color: "sol-green", desc: "Agent builds an unsigned tx; a human or Squads multisig signs." },
  { tier: "T2", color: "rewards", desc: "A scoped session key signs — locked to a {System, SAS, Memo} program allowlist, hard caps, identity check. Attest opt-in only." },
];

export default function Home() {
  return (
    <div className="flex flex-col flex-1">
      {/* ───────── nav ───────── */}
      <nav className="sticky top-0 z-40 border-b hairline bg-[color:var(--color-bg)]/80 backdrop-blur">
        <div className="mx-auto flex max-w-6xl items-center justify-between px-5 py-3.5">
          <a href="#top" className="flex items-center gap-2 font-semibold tracking-tight">
            <span className="inline-block h-2.5 w-2.5 rounded-sm sol-gradient-bg" aria-hidden />
            Palinurus
          </a>
          <div className="flex items-center gap-5 text-sm text-[color:var(--color-ink-muted)]">
            <a href="#wiring" className="hover:text-[color:var(--color-ink)] transition-colors">Wiring</a>
            <a href="#plugins" className="hover:text-[color:var(--color-ink)] transition-colors">Plugins</a>
            <a href="#custody" className="hover:text-[color:var(--color-ink)] transition-colors">Custody</a>
            <a href={REPO_URL} className="hover:text-[color:var(--color-ink)] transition-colors" target="_blank" rel="noreferrer">GitHub</a>
            <a href={PR_URL} className="rounded-md sol-gradient-bg px-3 py-1.5 font-medium text-black hover:opacity-90 transition-opacity" target="_blank" rel="noreferrer">PR #76</a>
          </div>
        </div>
      </nav>

      {/* ───────── hero ───────── */}
      <header id="top" className="hero-grid">
        <div className="mx-auto max-w-6xl px-5 py-24 sm:py-32">
          <div className="mx-auto max-w-3xl text-center">
            <span className="pill text-[color:var(--color-ink-muted)]">
              <span className="h-1.5 w-1.5 rounded-full bg-[color:var(--color-sol-green)]" aria-hidden />
              Superteam Brasil × ZeroClaw · Track C (DePIN)
            </span>
            <h1 className="mt-6 text-4xl font-semibold leading-[1.05] tracking-tight sm:text-6xl">
              The Solana DePIN node <br className="hidden sm:block" />
              that <span className="sol-gradient">talks</span>.
            </h1>
            <p className="mx-auto mt-6 max-w-2xl text-lg leading-relaxed text-[color:var(--color-ink-muted)]">
              A <span className="text-[color:var(--color-ink)]">$40 Raspberry Pi</span> running{" "}
              <a href="https://github.com/zeroclaw-labs/zeroclaw" className="text-[color:var(--color-ink)] underline decoration-[color:var(--color-border)] underline-offset-4 hover:decoration-[color:var(--color-ink)]" target="_blank" rel="noreferrer">ZeroClaw</a>{" "}
              becomes a Solana-attesting device — and the same agent watches your real Helium hotspot and pings you on Telegram the moment it goes dark.
              <br />
              <span className="text-[color:var(--color-ink)]">Agent proposes. Multisig disposes.</span> The agent never holds a main wallet key.
            </p>
            <div className="mt-9 flex flex-wrap items-center justify-center gap-3">
              <a href={PR_URL} className="rounded-lg sol-gradient-bg px-5 py-3 font-semibold text-black hover:opacity-90 transition-opacity" target="_blank" rel="noreferrer">
                View the PR →
              </a>
              <a href={BOUNTY_URL} className="rounded-lg border hairline px-5 py-3 font-medium text-[color:var(--color-ink)] hover:bg-[color:var(--color-panel)] transition-colors" target="_blank" rel="noreferrer">
                The bounty
              </a>
            </div>
            <div className="mt-12 grid grid-cols-2 gap-3 sm:grid-cols-4">
              {facts.map((f) => (
                <div key={f.k} className="rounded-xl border hairline bg-[color:var(--color-panel)]/60 px-4 py-3 text-left">
                  <div className="font-mono text-xl font-semibold text-[color:var(--color-ink)]">{f.k}</div>
                  <div className="text-xs text-[color:var(--color-ink-faint)]">{f.v}</div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </header>

      {/* ───────── wiring ───────── */}
      <section id="wiring" className="border-t hairline">
        <div className="mx-auto max-w-6xl px-5 py-20">
          <div className="mx-auto max-w-2xl text-center">
            <h2 className="text-3xl font-semibold tracking-tight sm:text-4xl">The wiring</h2>
            <p className="mt-4 text-[color:var(--color-ink-muted)]">
              Physical edge → ZeroClaw agent → Solana. Two plugins, one minimal substrate, every transaction signed on the cold path.
            </p>
          </div>
          <div className="mt-10 overflow-hidden rounded-2xl border hairline bg-[color:var(--color-bg-soft)] p-2 sm:p-4">
            {/* eslint-disable-next-line @next/next/no-img-element */}
            <img src="/wiring-diagram.svg" alt="Palinurus wiring diagram: Raspberry Pi running ZeroClaw hosts two WIT plugins (depin-attest, depin-rewards) that reach Solana and external APIs; custody tiers and the cold signing path shown." className="h-auto w-full" />
          </div>
        </div>
      </section>

      {/* ───────── plugins ───────── */}
      <section id="plugins" className="border-t hairline bg-[color:var(--color-bg-soft)]">
        <div className="mx-auto max-w-6xl px-5 py-20">
          <div className="mx-auto max-w-2xl text-center">
            <h2 className="text-3xl font-semibold tracking-tight sm:text-4xl">Two plugins, one substrate</h2>
            <p className="mt-4 text-[color:var(--color-ink-muted)]">
              Each a standalone <code className="font-mono text-sm text-[color:var(--color-ink)]">[workspace]</code> crate matching the canonical ZeroClaw plugin layout. Both depend on{" "}
              <a href={CORE_CRATES} className="text-[color:var(--color-ink)] underline underline-offset-4" target="_blank" rel="noreferrer">palinurus-core</a> from crates.io.
            </p>
          </div>
          <div className="mt-12 grid gap-6 md:grid-cols-2">
            {plugins.map((p) => (
              <article key={p.name} className="flex flex-col rounded-2xl border hairline bg-[color:var(--color-panel)] p-6">
                <div className="flex items-center justify-between">
                  <h3 className={`font-mono text-xl font-semibold text-${p.accent}`}>{p.name}</h3>
                  <span className={`pill text-${p.accent}`} style={{ borderColor: "currentColor" }}>{p.custody}</span>
                </div>
                <p className="mt-1 text-sm text-[color:var(--color-ink-faint)]">{p.tag}</p>
                <p className="mt-4 text-[color:var(--color-ink-muted)] leading-relaxed">{p.body}</p>
                <ul className="mt-5 flex flex-wrap gap-2">
                  {p.points.map((pt) => (
                    <li key={pt} className="rounded-md border hairline px-2.5 py-1 font-mono text-xs text-[color:var(--color-ink-muted)]">{pt}</li>
                  ))}
                </ul>
              </article>
            ))}
          </div>
        </div>
      </section>

      {/* ───────── custody ───────── */}
      <section id="custody" className="border-t hairline">
        <div className="mx-auto max-w-6xl px-5 py-20">
          <div className="grid gap-12 md:grid-cols-2">
            <div>
              <h2 className="text-3xl font-semibold tracking-tight sm:text-4xl">Custody is a first-class engineering problem</h2>
              <p className="mt-5 text-[color:var(--color-ink-muted)] leading-relaxed">
                An agent with a private key and an LLM in the loop is a hot wallet with a prompt-injection surface. Palinurus treats that seriously.
              </p>
              <p className="mt-4 text-[color:var(--color-ink-muted)] leading-relaxed">
                <span className="text-[color:var(--color-ink)]">The agent never holds a main wallet key.</span> It builds unsigned transactions; a human or a{" "}
                <a href="https://squads.so" className="text-[color:var(--color-ink)] underline underline-offset-4" target="_blank" rel="noreferrer">Squads multisig</a>{" "}
                signs. The durable nonce means a tx sitting in an approval queue for hours doesn&apos;t die from blockhash expiry.
              </p>
              <p className="mt-4 text-[color:var(--color-ink-muted)] leading-relaxed">
                Where autonomous signing is opt-in (depin-attest T2), it&apos;s locked to a program allowlist that{" "}
                <span className="text-[color:var(--color-ink)]">blocks value transfer</span>, a session key that holds cents, and a fail-closed prompt-injection test transcript in the README.
              </p>
            </div>
            <div className="flex flex-col gap-4">
              {tiers.map((t) => (
                <div key={t.tier} className="flex gap-4 rounded-xl border hairline bg-[color:var(--color-panel)] p-5">
                  <div className={`font-mono text-2xl font-bold text-${t.color}`}>{t.tier}</div>
                  <div className="text-[color:var(--color-ink-muted)]">{t.desc}</div>
                </div>
              ))}
              <p className="px-1 text-xs text-[color:var(--color-ink-faint)]">
                Claim moves value ⇒ depin-rewards stays T0/T1 (multisig). T2 is attestation-only — blast radius = fake attestations, not theft.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* ───────── demo + cta ───────── */}
      <section id="demo" className="border-t hairline bg-[color:var(--color-bg-soft)]">
        <div className="mx-auto max-w-6xl px-5 py-20 text-center">
          <h2 className="text-3xl font-semibold tracking-tight sm:text-4xl">See it run</h2>
          <p className="mx-auto mt-4 max-w-2xl text-[color:var(--color-ink-muted)]">
            The demo watches a real public Helium hotspot — <span className="font-mono text-[color:var(--color-ink)]">Fit Pine Capybara</span> — that earned 0.02 HNT then went offline. The watch action fires a real Telegram alert to a real phone. No mock, no slide deck.
          </p>
          <div className="mx-auto mt-10 max-w-2xl overflow-hidden rounded-2xl border hairline bg-[color:var(--color-bg)] p-10">
            <div className="font-mono text-sm text-[color:var(--color-ink-faint)]">demo video — recording now</div>
            <div className="mt-3 font-semibold text-[color:var(--color-ink)]">≤ 3 minutes · real agent · real Telegram · real Solana</div>
            <p className="mt-2 text-sm text-[color:var(--color-ink-muted)]">
              The full recording guide lives at{" "}
              <a href={`${REPO_URL}/blob/main/docs/demo-recording-guide.md`} className="text-[color:var(--color-ink)] underline underline-offset-4" target="_blank" rel="noreferrer">docs/demo-recording-guide.md</a>.
            </p>
          </div>
        </div>
      </section>

      {/* ───────── footer ───────── */}
      <footer className="mt-auto border-t hairline">
        <div className="mx-auto flex max-w-6xl flex-col items-center justify-between gap-4 px-5 py-10 text-sm text-[color:var(--color-ink-faint)] sm:flex-row">
          <div className="flex items-center gap-2">
            <span className="inline-block h-2 w-2 rounded-sm sol-gradient-bg" aria-hidden />
            <span>Palinurus · MIT licensed · built by RECTOR-LABS</span>
          </div>
          <div className="flex items-center gap-5">
            <a href={REPO_URL} className="hover:text-[color:var(--color-ink)]" target="_blank" rel="noreferrer">GitHub</a>
            <a href={CORE_CRATES} className="hover:text-[color:var(--color-ink)]" target="_blank" rel="noreferrer">crates.io</a>
            <a href={PR_URL} className="hover:text-[color:var(--color-ink)]" target="_blank" rel="noreferrer">PR #76</a>
            <a href={BOUNTY_URL} className="hover:text-[color:var(--color-ink)]" target="_blank" rel="noreferrer">Superteam</a>
          </div>
        </div>
      </footer>
    </div>
  );
}