//! Host tests for the `durable_nonce` module (slice 5 of palinurus-core).
//!
//! Solves blockhash expiry: a tx built now can sit in an approval queue past the
//! blockhash TTL. A durable-nonce tx uses the nonce account's stored
//! `DurableNonce` as `recent_blockhash` + an `Advance` ix first, so it never
//! expires. Consensus-critical pieces are verified byte-for-byte against
//! `solana_program`:
//!   - `nonce_advance_ix` / `nonce_authorize_ix` vs
//!     `solana_program::system_instruction::{advance,authorize}_nonce_account`.
//!   - `parse_nonce_account` vs `bincode::serialize(&Versions::new(State::Initialized(..)))`.
//!   - `build_with_durable_nonce` vs `MessageV0::try_compile([advance, ..user], &[], Hash(nonce))`.

use palinurus_core::base58::Pubkey;
use palinurus_core::durable_nonce::{
    build_with_durable_nonce, nonce_advance_ix, nonce_authorize_ix, parse_nonce_account,
    NonceError, NonceState, NonceVersion,
};
use palinurus_core::versioned_tx::{serialize, Instruction};
use palinurus_core::borsh::CreateAttestationIxData;
use borsh::to_vec;

// ---- Oracle helpers (solana_program) ----
fn sol_pk(b: [u8; 32]) -> solana_program::pubkey::Pubkey { solana_program::pubkey::Pubkey::new_from_array(b) }
fn sol_hash(b: [u8; 32]) -> solana_program::hash::Hash { solana_program::hash::Hash::new_from_array(b) }
fn sol_meta_buf(am: &palinurus_core::versioned_tx::AccountMeta) -> solana_program::instruction::AccountMeta {
    solana_program::instruction::AccountMeta { pubkey: sol_pk(am.pubkey.to_bytes()), is_signer: am.is_signer, is_writable: am.is_writable }
}
fn sol_ix(ix: &Instruction) -> solana_program::instruction::Instruction {
    solana_program::instruction::Instruction { program_id: sol_pk(ix.program_id.to_bytes()), accounts: ix.accounts.iter().map(sol_meta_buf).collect(), data: ix.data.clone() }
}

fn pk(b: u8) -> Pubkey { Pubkey::from_bytes([b; 32]) }
fn nonce_value() -> [u8; 32] { [0xAB; 32] }

// ---- nonce_advance_ix ----

#[test]
fn advance_ix_matches_solana_system_instruction_byte_for_byte() {
    let nonce = pk(0x10);
    let authority = pk(0x20);
    let mine = nonce_advance_ix(nonce, authority);
    let ref_ = solana_program::system_instruction::advance_nonce_account(&sol_pk(nonce.to_bytes()), &sol_pk(authority.to_bytes()));

    assert_eq!(mine.program_id, Pubkey::SYSTEM, "advance program = system");
    assert_eq!(mine.data, ref_.data, "advance ix data must match solana_program");
    assert_eq!(mine.data, vec![0x04, 0x00, 0x00, 0x00], "AdvanceNonceAccount = variant 4, u32 LE");
    assert_eq!(mine.accounts.len(), ref_.accounts.len(), "account count");
    // accounts: [nonce writable non-signer, recent_blockhashes sysvar readonly non-signer, authority readonly signer]
    assert_eq!(mine.accounts.len(), 3);
    assert_eq!(mine.accounts[0].pubkey, nonce);
    assert!(mine.accounts[0].is_writable && !mine.accounts[0].is_signer);
    assert_eq!(mine.accounts[1].pubkey.to_string(), "SysvarRecentB1ockHashes11111111111111111111");
    assert!(!mine.accounts[1].is_writable && !mine.accounts[1].is_signer);
    assert_eq!(mine.accounts[2].pubkey, authority);
    assert!(!mine.accounts[2].is_writable && mine.accounts[2].is_signer);
    // Full field-by-field equality with the oracle.
    for (i, (m, r)) in mine.accounts.iter().zip(ref_.accounts.iter()).enumerate() {
        assert_eq!(m.pubkey.to_bytes(), r.pubkey.to_bytes(), "account {i} pubkey");
        assert_eq!(m.is_signer, r.is_signer, "account {i} is_signer");
        assert_eq!(m.is_writable, r.is_writable, "account {i} is_writable");
    }
    assert_eq!(mine.program_id.to_bytes(), ref_.program_id.to_bytes());
}

// ---- nonce_authorize_ix ----

#[test]
fn authorize_ix_matches_solana_system_instruction_byte_for_byte() {
    let nonce = pk(0x10);
    let authority = pk(0x20);
    let new_authority = pk(0x30);
    let mine = nonce_authorize_ix(nonce, authority, new_authority);
    let ref_ = solana_program::system_instruction::authorize_nonce_account(
        &sol_pk(nonce.to_bytes()),
        &sol_pk(authority.to_bytes()),
        &sol_pk(new_authority.to_bytes()),
    );
    assert_eq!(mine.program_id.to_bytes(), ref_.program_id.to_bytes());
    assert_eq!(mine.data, ref_.data, "authorize ix data must match solana_program");
    // AuthorizeNonceAccount(Pubkey) = variant 7, u32 LE + 32B pubkey
    let mut expected = vec![0x07, 0x00, 0x00, 0x00];
    expected.extend_from_slice(new_authority.as_bytes());
    assert_eq!(mine.data, expected);
    assert_eq!(mine.accounts.len(), ref_.accounts.len());
    for (i, (m, r)) in mine.accounts.iter().zip(ref_.accounts.iter()).enumerate() {
        assert_eq!(m.pubkey.to_bytes(), r.pubkey.to_bytes(), "account {i} pubkey");
        assert_eq!(m.is_signer, r.is_signer, "account {i} is_signer");
        assert_eq!(m.is_writable, r.is_writable, "account {i} is_writable");
    }
}

// ---- parse_nonce_account ----

#[test]
fn parse_initialized_nonce_account_matches_oracle() {
    use solana_program::nonce::state::{DurableNonce, State, Versions};
    let authority = pk(0x20);
    let blockhash = sol_hash([0x11; 32]);
    let durable_nonce = DurableNonce::from_blockhash(&blockhash);
    let expected_nonce_bytes = durable_nonce.as_hash().to_bytes();
    let lamports = 5000u64;
    let versions = Versions::new(State::new_initialized(
        &sol_pk(authority.to_bytes()),
        durable_nonce,
        lamports,
    ));
    let data = bincode::serialize(&versions).expect("oracle: serialize Versions");

    let parsed = parse_nonce_account(&data).expect("parse");
    assert!(matches!(parsed.version, NonceVersion::Current), "Versions::new -> Current");
    match parsed.state {
        NonceState::Initialized(d) => {
            assert_eq!(d.authority, authority);
            assert_eq!(d.durable_nonce, expected_nonce_bytes);
            assert_eq!(d.lamports_per_signature, lamports);
        }
        NonceState::Uninitialized => panic!("expected Initialized"),
    }
}

#[test]
fn parse_uninitialized_nonce_account() {
    use solana_program::nonce::state::{State, Versions};
    let versions = Versions::new(State::Uninitialized);
    let data = bincode::serialize(&versions).expect("oracle: serialize Versions (uninitialized)");
    // Versions::new -> Current(Uninitialized) -> 8 bytes; pad to 80 (account alloc).
    let mut buf = data.clone();
    buf.resize(80, 0);
    let parsed = parse_nonce_account(&buf).expect("parse");
    assert!(matches!(parsed.version, NonceVersion::Current));
    assert!(matches!(parsed.state, NonceState::Uninitialized));
}

#[test]
fn parse_nonce_account_rejects_short_buffer() {
    let short = vec![0u8; 7];
    assert_eq!(parse_nonce_account(&short), Err(NonceError::InvalidLength));
}

// ---- build_with_durable_nonce ----

fn sas_user_ix(payer: Pubkey, authority: Pubkey, attestation: Pubkey) -> Instruction {
    // A realistic depin-attest user instruction (SAS create_attestation).
    let ix_data = CreateAttestationIxData::new(authority, b"temp=24.7C".to_vec(), 1753000000);
    Instruction {
        program_id: Pubkey::SAS,
        accounts: vec![
            palinurus_core::versioned_tx::AccountMeta::signer_writable(payer),
            palinurus_core::versioned_tx::AccountMeta::signer_readonly(authority),
            palinurus_core::versioned_tx::AccountMeta::readonly(pk(0x03)), // credential
            palinurus_core::versioned_tx::AccountMeta::readonly(pk(0x04)), // schema
            palinurus_core::versioned_tx::AccountMeta::writable(attestation),
            palinurus_core::versioned_tx::AccountMeta::readonly(Pubkey::SYSTEM),
        ],
        data: to_vec(&ix_data).unwrap(),
    }
}

#[test]
fn build_with_durable_nonce_matches_oracle_byte_for_byte() {
    let payer = pk(0x01);
    let authority = pk(0x20);
    let nonce_account = pk(0x10);
    let attestation = pk(0x05);
    let nonce = nonce_value();
    let user_ixs = vec![sas_user_ix(payer, authority, attestation)];

    let mine = serialize(&build_with_durable_nonce(&user_ixs, payer, nonce_account, nonce, authority));

    // Oracle: [advance_sol, ..user_sol] compiled with Hash(nonce) as blockhash.
    let advance_sol = solana_program::system_instruction::advance_nonce_account(
        &sol_pk(nonce_account.to_bytes()),
        &sol_pk(authority.to_bytes()),
    );
    let mut all_sol = vec![advance_sol];
    all_sol.extend(user_ixs.iter().map(sol_ix));
    let v0 = solana_program::message::v0::Message::try_compile(
        &sol_pk(payer.to_bytes()),
        &all_sol,
        &[],
        sol_hash(nonce),
    )
    .expect("oracle: try_compile");
    let msg_bytes = bincode::serialize(&solana_program::message::VersionedMessage::V0(v0)).unwrap();
    let mut ref_tx = vec![0x00]; // zero signatures
    ref_tx.extend_from_slice(&msg_bytes);

    assert_eq!(mine, ref_tx, "durable-nonce tx must match oracle byte-for-byte");
}

#[test]
fn build_with_durable_nonce_recent_blockhash_is_the_nonce_and_advance_is_first() {
    let payer = pk(0x01);
    let authority = pk(0x20);
    let nonce_account = pk(0x10);
    let nonce = nonce_value();
    let user_ixs = vec![sas_user_ix(payer, authority, pk(0x05))];
    let tx = build_with_durable_nonce(&user_ixs, payer, nonce_account, nonce, authority);

    assert_eq!(tx.message.recent_blockhash, nonce, "recent_blockhash must be the durable nonce");
    assert_eq!(tx.message.instructions.len(), 2, "advance + 1 user ix");
    // First compiled instruction is the Advance ix (system program).
    let advance_ci = &tx.message.instructions[0];
    let advance_program = tx.message.account_keys[advance_ci.program_id_index as usize];
    assert_eq!(advance_program, Pubkey::SYSTEM, "first ix program = system (Advance)");
    assert_eq!(advance_ci.data, vec![0x04, 0x00, 0x00, 0x00], "first ix data = AdvanceNonceAccount");
    // The nonce account + authority + sysvar are in the account keys.
    let keys: Vec<_> = tx.message.account_keys.iter().collect();
    assert!(keys.contains(&&nonce_account), "nonce account present in keys");
    assert!(keys.contains(&&authority), "authority present in keys");
}

#[test]
fn build_with_durable_nonce_requires_two_signers_payer_and_authority() {
    let payer = pk(0x01);
    let authority = pk(0x20);
    let nonce = nonce_value();
    let user_ixs = vec![sas_user_ix(payer, authority, pk(0x05))];
    let tx = build_with_durable_nonce(&user_ixs, payer, pk(0x10), nonce, authority);
    assert_eq!(tx.message.header.num_required_signatures, 2, "payer + authority both sign");
}