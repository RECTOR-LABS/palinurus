// Independent cross-check: derives the same PDA as `src/pda.rs` using the
// canonical Solana reference implementation (@solana/web3.js), so we can paste
// the result into `src/pda.rs`'s `matches_solana_web3_js_reference` test.
//
// Run:  node tools/verify-pda.mjs
// (installs @solana/web3.js on first run via the local node_modules)

import { PublicKey } from "@solana/web3.js";

// Must match `test_program_id()` in src/pda.rs: 32 × 0x42.
const programIdBytes = Buffer.alloc(32, 0x42);
const programId = new PublicKey(programIdBytes);

const seeds = [Buffer.from("palinurus"), Buffer.from("depin-attest")];
const [pda, bump] = PublicKey.findProgramAddressSync(seeds, programId);

console.log("seeds:        ", seeds.map((s) => s.toString()));
console.log("program_id:   ", programId.toBase58(), "(32 × 0x42)");
console.log("pda (base58): ", pda.toBase58());
console.log("bump:         ", bump);

// Also a single-seed case used by the property tests, for an extra eyeball check.
const [pda1, bump1] = PublicKey.findProgramAddressSync(
  [Buffer.from("palinurus")],
  programId,
);
console.log("--- single seed [palinurus] ---");
console.log("pda (base58): ", pda1.toBase58());
console.log("bump:         ", bump1);