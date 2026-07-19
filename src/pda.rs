//! Solana Program Derived Address (PDA) derivation.
//!
//! A hand-rolled, `wasm32-wasip2`-friendly reimplementation of
//! `solana_program::Pubkey::find_program_address` using only `sha2` (SHA-256)
//! and `curve25519-dalek` (the off-curve check), so it compiles inside a WIT
//! component where `solana-sdk`/`solana-program` cannot.
//!
//! ## Algorithm (matches `solana_program` exactly)
//!
//! A PDA is a 32-byte value that is **not** a valid ed25519 public key (i.e. it
//! is "off-curve", so it has no associated private key and cannot be a signer).
//! Canonical derivation (`solana_program::Pubkey::try_find_program_address`)
//! appends the bump as the last seed, then the program id, then the `PDA_MARKER`
//! domain separator, and tries bump seeds from 255 down to 1, returning the
//! **highest** bump whose hash is off-curve:
//!
//! ```text
//! for bump in 255..=1 (descending):
//!     h = sha256(seed_1 || seed_2 || ... || bump || program_id || PDA_MARKER)
//!     if h is NOT a valid ed25519 compressed point (off-curve):
//!         return (h, bump)
//! ```
//!
//! ~50% of 32-byte hashes are off-curve, so a result is always found — the
//! no-result branch is practically unreachable (mirrors `find_program_address`
//! panicking on `try_find_program_address`'s `None`).
//!
//! ## Why this matters for Palinurus
//!
//! Track C's `depin-attest` plugin commits sensor readings to Solana's
//! Attestation Service (SAS). Every SAS account (Credential, Schema, Attestation)
//! is a PDA under the SAS program, so the plugin must derive PDAs to build the
//! `create_attestation` instruction. Doing this inside the WASM sandbox — without
//! `solana-program` — is the single hardest piece of the "solana-sdk won't
//! compile for wasm" trap this bounty warns about. This module is the de-risk
//! spike for that: if `curve25519-dalek` + `sha2` compile to `wasm32-wasip2` and
//! the derivation matches the reference algorithm, SAS-primary is confirmed.

use curve25519_dalek::edwards::CompressedEdwardsY;
use sha2::{Digest, Sha256};

/// A Solana public key / PDA is 32 bytes.
pub type PubkeyBytes = [u8; 32];

/// Domain-separation marker appended to every PDA derivation hash, matching
/// `solana_program::pubkey::PDA_MARKER` (v1.18 classic and v2.x both use it).
/// The canonical on-chain derivation is
/// `sha256(seed_1 || ... || bump || program_id || PDA_MARKER)` — the bump is the
/// last seed (tried 255 → 1) and the marker follows the program id.
const PDA_MARKER: &[u8; 21] = b"ProgramDerivedAddress";

/// Returns `true` if `bytes` are a valid ed25519 compressed point (on-curve).
///
/// A PDA must be **off-curve** (`is_on_curve == false`) so it has no associated
/// private key. This mirrors `solana_program::Pubkey::is_on_curve`, which uses
/// `curve25519_dalek::edwards::CompressedEdwardsY::decompress`.
pub fn is_on_curve(bytes: &PubkeyBytes) -> bool {
    CompressedEdwardsY(*bytes).decompress().is_some()
}

/// Derive the canonical (highest-bump) PDA for `seeds` under `program_id`.
///
/// Mirrors `solana_program::Pubkey::find_program_address`: the bump is appended
/// as the last seed, then the program id and `PDA_MARKER` follow; bump seeds are
/// tried 255 → 1 (descending) and the first (highest) off-curve hash wins.
///
/// Returns `(pda_bytes, bump_seed)`. Panics only if no off-curve hash is found
/// across 255 bumps — practically unreachable (~0.5^255 probability).
pub fn find_program_address(seeds: &[&[u8]], program_id: &PubkeyBytes) -> (PubkeyBytes, u8) {
    // Matches solana_program::Pubkey::try_find_program_address: bump is the
    // last seed, then program_id, then the PDA_MARKER domain separator. Bump
    // tries 255 → 1 (solana's loop is `0..u8::MAX` starting at u8::MAX).
    for bump in (1u8..=255).rev() {
        let mut hasher = Sha256::new();
        for seed in seeds {
            hasher.update(seed);
        }
        hasher.update([bump]);
        hasher.update(program_id);
        hasher.update(PDA_MARKER);
        let hash: PubkeyBytes = hasher.finalize().into();
        if !is_on_curve(&hash) {
            return (hash, bump);
        }
    }
    // See crate docs: ~50% of hashes are off-curve, so this is unreachable.
    unreachable!("no off-curve PDA found across 255 bumps (probability ~0.5^255)");
}

#[cfg(test)]
mod solana_oracle {
    use super::*;
    /// The consensus-critical on-chain derivation (`solana_program::Pubkey::
    /// find_program_address`). If our hand-rolled `find_program_address` matches
    /// this byte-for-byte, it produces real on-chain-valid PDAs.
    #[test]
    fn matches_canonical_solana_program_derivation_two_seeds() {
        let program_id = solana_program::pubkey::Pubkey::new_from_array([0x42u8; 32]);
        let seeds: &[&[u8]] = &[b"palinurus", b"depin-attest"];

        let (ref_pda, ref_bump) =
            solana_program::pubkey::Pubkey::find_program_address(seeds, &program_id);
        let ref_b58 = bs58::encode(ref_pda.to_bytes()).into_string();

        let (my_pda, my_bump) = find_program_address(seeds, &program_id.to_bytes());
        let my_b58 = bs58::encode(my_pda).into_string();

        eprintln!(
            "[oracle] solana-program ref: {ref_b58} bump {ref_bump} | ours: {my_b58} bump {my_bump}"
        );
        assert_eq!(my_b58, ref_b58, "PDA must match canonical solana-program derivation");
        assert_eq!(my_bump, ref_bump, "bump must match canonical solana-program derivation");
    }

    #[test]
    fn matches_canonical_solana_program_derivation_single_seed() {
        let program_id = solana_program::pubkey::Pubkey::new_from_array([0x42u8; 32]);
        let seeds: &[&[u8]] = &[b"palinurus"];

        let (ref_pda, ref_bump) =
            solana_program::pubkey::Pubkey::find_program_address(seeds, &program_id);
        let ref_b58 = bs58::encode(ref_pda.to_bytes()).into_string();

        let (my_pda, my_bump) = find_program_address(seeds, &program_id.to_bytes());
        let my_b58 = bs58::encode(my_pda).into_string();

        eprintln!(
            "[oracle] solana-program ref: {ref_b58} bump {ref_bump} | ours: {my_b58} bump {my_bump}"
        );
        assert_eq!(my_b58, ref_b58, "PDA must match canonical solana-program derivation");
        assert_eq!(my_bump, ref_bump, "bump must match canonical solana-program derivation");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed, arbitrary program id (32 × 0x42) used across the property tests.
    /// Its base58 form is computed in the cross-check test below.
    fn test_program_id() -> PubkeyBytes {
        [0x42; 32]
    }

    #[test]
    fn pda_is_off_curve() {
        let program_id = test_program_id();
        let (pda, _bump) = find_program_address(&[b"palinurus", b"depin-attest"], &program_id);
        assert!(
            !is_on_curve(&pda),
            "derived PDA must be off-curve (no associated private key)"
        );
    }

    #[test]
    fn pda_is_deterministic() {
        let program_id = test_program_id();
        let (pda1, bump1) = find_program_address(&[b"palinurus"], &program_id);
        let (pda2, bump2) = find_program_address(&[b"palinurus"], &program_id);
        assert_eq!(pda1, pda2, "same seeds must yield the same PDA");
        assert_eq!(bump1, bump2, "same seeds must yield the same bump");
    }

    #[test]
    fn different_seeds_yield_different_pdas() {
        let program_id = test_program_id();
        let (pda_a, _) = find_program_address(&[b"palinurus"], &program_id);
        let (pda_b, _) = find_program_address(&[b"oracle"], &program_id);
        assert_ne!(pda_a, pda_b, "different seeds must yield different PDAs");
    }

    #[test]
    fn different_programs_yield_different_pdas() {
        let seeds: &[&[u8]] = &[b"palinurus"];
        let prog_a = [0x42; 32];
        let prog_b = [0x43; 32];
        let (pda_a, _) = find_program_address(seeds, &prog_a);
        let (pda_b, _) = find_program_address(seeds, &prog_b);
        assert_ne!(
            pda_a, pda_b,
            "same seeds under different programs must yield different PDAs"
        );
    }

    #[test]
    fn bump_is_the_highest_off_curve() {
        // find_program_address returns the highest bump (255→0 search) that is
        // off-curve. Verify every bump strictly greater than the returned one is
        // on-curve (otherwise that higher bump would have been returned), and the
        // returned bump's hash equals the PDA.
        let program_id = test_program_id();
        let (pda, bump) = find_program_address(&[b"palinurus"], &program_id);

        for higher_u16 in ((bump as u16) + 1)..=255u16 {
            let higher = higher_u16 as u8;
            let mut hasher = Sha256::new();
            hasher.update(b"palinurus");
            hasher.update([higher]);
            hasher.update(program_id);
            hasher.update(PDA_MARKER);
            let h: PubkeyBytes = hasher.finalize().into();
            assert!(
                is_on_curve(&h),
                "bump {higher} is higher than returned bump {bump} but is off-curve — search order is wrong"
            );
        }

        // The returned bump's hash must equal the PDA.
        let mut hasher = Sha256::new();
        hasher.update(b"palinurus");
        hasher.update([bump]);
        hasher.update(program_id);
        hasher.update(PDA_MARKER);
        let h: PubkeyBytes = hasher.finalize().into();
        assert_eq!(pda, h, "PDA must equal sha256(seeds || bump || program || PDA_MARKER)");
    }

    #[test]
    fn empty_seeds_still_derive_an_off_curve_pda() {
        // Edge case: no seeds. Mirrors solana find_program_address(&[], &prog).
        let program_id = test_program_id();
        let (pda, _bump) = find_program_address(&[], &program_id);
        assert!(!is_on_curve(&pda), "empty-seed PDA must still be off-curve");
    }

    /// Cross-check against the canonical Solana reference implementation
    /// (`@solana/web3.js` `PublicKey.findProgramAddressSync`). The expected
    /// `(base58, bump)` constants below were produced by an independent Node.js
    /// script (`tools/verify-pda.mjs`, run 2026-07-19) using `@solana/web3.js`
    /// for the same seeds + program id (32 × 0x42, base58
    /// `5TeWSsjg2gbxCyWVniXeCmwM7UtHTCK7svzJr5xYJzHf`). If this test passes, our
    /// hand-rolled derivation matches the reference exactly — SAS-primary is
    /// confirmed feasible (PDA derivation works inside a WIT component without
    /// `solana-program`).
    #[test]
    fn matches_solana_web3_js_reference_two_seeds() {
        // seeds = ["palinurus", "depin-attest"], program_id = 32 × 0x42
        let expected_pda_b58: &str = "Et5CGkEYE5YqNYbReBAaBTmkZ8txz2D4kcjCVhWcvT2p";
        let expected_bump: u8 = 252;

        let program_id = test_program_id();
        let (pda, bump) = find_program_address(&[b"palinurus", b"depin-attest"], &program_id);
        let got_b58 = bs58::encode(pda).into_string();
        assert_eq!(got_b58, expected_pda_b58, "PDA base58 must match @solana/web3.js reference");
        assert_eq!(bump, expected_bump, "bump must match @solana/web3.js reference");
    }

    /// Second reference vector (single seed) from the same `@solana/web3.js` run.
    #[test]
    fn matches_solana_web3_js_reference_single_seed() {
        // seeds = ["palinurus"], program_id = 32 × 0x42
        let expected_pda_b58: &str = "2Jbijq1B93p6CpXv2yz4hwoz1gn3hQLFHjy7zPobWWgo";
        let expected_bump: u8 = 255;

        let program_id = test_program_id();
        let (pda, bump) = find_program_address(&[b"palinurus"], &program_id);
        let got_b58 = bs58::encode(pda).into_string();
        assert_eq!(got_b58, expected_pda_b58, "PDA base58 must match @solana/web3.js reference");
        assert_eq!(bump, expected_bump, "bump must match @solana/web3.js reference");
    }
}