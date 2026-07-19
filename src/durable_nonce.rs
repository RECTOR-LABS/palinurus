//! Durable transaction nonces — the blockhash-expiry fix.
//!
//! An agent builds a tx → it drops into an approval queue → 5 min later the
//! blockhash is dead and the tx can't land. A **durable nonce** replaces the
//! `recent_blockhash` with a value stored in an on-chain nonce account, so the
//! tx never expires. Each submit advances the stored nonce, which is how a
//! replayed signed durable-nonce tx is rejected.
//!
//! ## Usage (T1 unsigned)
//!
//! The plugin reads the nonce account from chain (`rpc.get_account_info`),
//! parses it with [`parse_nonce_account`] to extract the stored `durable_nonce`
//! and `authority`, then [`build_with_durable_nonce`] prepends an `Advance`
//! instruction and uses the stored nonce as `recent_blockhash`. A human / Squads
//! multisig (the authority) signs and submits. The tx lives until they sign.
//!
//! ## Consensus-critical — oracle-verified
//!
//! The `Advance` / `Authorize` instruction bytes and the `NonceAccount`
//! (`Versions`) byte layout are verified byte-for-byte against
//! `solana_program::system_instruction` and `solana_program::nonce::state` in
//! `tests/durable_nonce.rs` (same rigor as PDA + versioned_tx).
//!
//! ## Nonce account layout (initialized, `Current` version — 80 bytes)
//!
//! `[u32 LE version=1 (Current)] [u32 LE state=1 (Initialized)] [32B authority]
//! [32B durable_nonce] [u64 LE lamports_per_signature]`
//!
//! Enum variant tags are `u32 LE` (confirmed by `State::size() == 80`:
//! 4 + 4 + 32 + 32 + 8). `Legacy` (version=0) durable nonces are invalid for
//! versioned transactions; the parser still returns them so the plugin can
//! reject with a clear error.

use crate::base58::Pubkey;
use crate::versioned_tx::{build_unsigned, AccountMeta, Blockhash, Instruction, VersionedTransaction};

/// A durable nonce value — the 32-byte Hash stored in a nonce account, used as
/// the `recent_blockhash` of a durable-nonce transaction.
pub type DurableNonce = Blockhash;

/// Nonce account version (the `Versions` enum tag).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NonceVersion {
    Legacy,
    Current,
}

/// Parsed nonce-account state.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NonceState {
    Uninitialized,
    Initialized(NonceData),
}

/// The initialized nonce-account data (authority + stored nonce + fee).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NonceData {
    /// The account that must sign transactions using this nonce (and the
    /// `Advance` instruction).
    pub authority: Pubkey,
    /// The durable nonce — use as `recent_blockhash`.
    pub durable_nonce: DurableNonce,
    /// Fee per signature (from the `FeeCalculator` stored alongside the nonce).
    pub lamports_per_signature: u64,
}

/// A parsed nonce account.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NonceAccount {
    pub version: NonceVersion,
    pub state: NonceState,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NonceError {
    /// The account data is too short to contain the version + state tags (or
    /// too short for an Initialized account's 80 bytes).
    InvalidLength,
    /// Unknown `Versions` variant tag (not 0=Legacy or 1=Current).
    UnsupportedVersion(u32),
    /// Unknown `State` variant tag (not 0=Uninitialized or 1=Initialized).
    BadStateTag(u32),
}

/// The `RecentBlockhashes` sysvar account — required (readonly, non-signer) by
/// the `Advance` instruction. Bytes from `@solana/web3.js` (base58
/// `SysvarRecentB1ockHashes11111111111111111111`).
pub const RECENT_BLOCKHASHES_ID: Pubkey = Pubkey::from_bytes([
    0x06, 0xa7, 0xd5, 0x17, 0x19, 0x2c, 0x56, 0x8e, 0xe0, 0x8a, 0x84, 0x5f, 0x73, 0xd2, 0x97, 0x88,
    0xcf, 0x03, 0x5c, 0x31, 0x45, 0xb2, 0x1a, 0xb3, 0x44, 0xd8, 0x06, 0x2e, 0xa9, 0x40, 0x00, 0x00,
]);

/// `SystemInstruction::AdvanceNonceAccount` = variant index 4, bincode-serialized
/// as a `u32 LE` tag with no payload → `[0x04, 0x00, 0x00, 0x00]`.
const ADVANCE_NONCE_ACCOUNT_IX_DATA: [u8; 4] = [0x04, 0x00, 0x00, 0x00];

/// `SystemInstruction::AuthorizeNonceAccount(Pubkey)` = variant index 7, bincode
/// as `u32 LE` tag + 32-byte pubkey.
const AUTHORIZE_NONCE_ACCOUNT_DISCRIMINATOR: [u8; 4] = [0x07, 0x00, 0x00, 0x00];

/// Build the System program `AdvanceNonceAccount` instruction.
///
/// Accounts (in order): `[nonce (writable, non-signer), RecentBlockhashes sysvar
/// (readonly, non-signer), authority (readonly, signer)]`. Data =
/// `[0x04,0x00,0x00,0x00]`. Verified byte-for-byte against
/// `solana_program::system_instruction::advance_nonce_account`.
pub fn nonce_advance_ix(nonce: Pubkey, authority: Pubkey) -> Instruction {
    Instruction {
        program_id: Pubkey::SYSTEM,
        accounts: vec![
            AccountMeta::writable(nonce),
            AccountMeta::readonly(RECENT_BLOCKHASHES_ID),
            AccountMeta::signer_readonly(authority),
        ],
        data: ADVANCE_NONCE_ACCOUNT_IX_DATA.to_vec(),
    }
}

/// Build the System program `AuthorizeNonceAccount(new_authority)` instruction.
///
/// Accounts: `[nonce (writable, non-signer), authority (readonly, signer)]`.
/// Data = `[0x07,0x00,0x00,0x00]` + 32-byte `new_authority`. Verified byte-for-byte
/// against `solana_program::system_instruction::authorize_nonce_account`.
pub fn nonce_authorize_ix(nonce: Pubkey, authority: Pubkey, new_authority: Pubkey) -> Instruction {
    let mut data = AUTHORIZE_NONCE_ACCOUNT_DISCRIMINATOR.to_vec();
    data.extend_from_slice(new_authority.as_bytes());
    Instruction {
        program_id: Pubkey::SYSTEM,
        accounts: vec![
            AccountMeta::writable(nonce),
            AccountMeta::signer_readonly(authority),
        ],
        data,
    }
}

/// Parse a nonce account's data bytes into a [`NonceAccount`].
///
/// Matches `bincode::deserialize::<solana_nonce::state::Versions>`: `u32 LE`
/// version tag (0=Legacy, 1=Current) + `u32 LE` state tag (0=Uninitialized,
/// 1=Initialized) + (if Initialized) 32B authority + 32B durable_nonce +
/// `u64 LE` lamports_per_signature.
pub fn parse_nonce_account(data: &[u8]) -> Result<NonceAccount, NonceError> {
    if data.len() < 8 {
        return Err(NonceError::InvalidLength);
    }
    let version_tag = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let version = match version_tag {
        0 => NonceVersion::Legacy,
        1 => NonceVersion::Current,
        other => return Err(NonceError::UnsupportedVersion(other)),
    };
    let state_tag = u32::from_le_bytes(data[4..8].try_into().unwrap());
    let state = match state_tag {
        0 => NonceState::Uninitialized,
        1 => {
            if data.len() < 80 {
                return Err(NonceError::InvalidLength);
            }
            let authority = Pubkey::from_bytes(data[8..40].try_into().unwrap());
            let durable_nonce = data[40..72].try_into().unwrap();
            let lamports_per_signature = u64::from_le_bytes(data[72..80].try_into().unwrap());
            NonceState::Initialized(NonceData {
                authority,
                durable_nonce,
                lamports_per_signature,
            })
        }
        other => return Err(NonceError::BadStateTag(other)),
    };
    Ok(NonceAccount { version, state })
}

/// Build an unsigned durable-nonce versioned transaction (T1): `Advance` ix
/// first, then `user_ixs`, with `nonce` (the stored `DurableNonce`) as
/// `recent_blockhash`. The `authority` + `payer` sign later (human / Squads
/// multisig). The tx does not expire while it waits.
///
/// Verified byte-for-byte against
/// `solana_program::message::v0::Message::try_compile([advance, ..user], &[], Hash(nonce))`.
pub fn build_with_durable_nonce(
    user_ixs: &[Instruction],
    payer: Pubkey,
    nonce_account: Pubkey,
    nonce: DurableNonce,
    authority: Pubkey,
) -> VersionedTransaction {
    let advance = nonce_advance_ix(nonce_account, authority);
    let mut all = Vec::with_capacity(user_ixs.len() + 1);
    all.push(advance);
    all.extend(user_ixs.iter().cloned());
    build_unsigned(&all, payer, nonce)
}