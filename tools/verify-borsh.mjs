// Independent cross-check: encodes the SAS `create_attestation` instruction DATA
// using the canonical sas-lib Codama-generated codec (@solana/kit), so we can
// paste the resulting bytes into `tests/borsh.rs` as a ground-truth vector.
//
// Run:  node tools/verify-borsh.mjs
// (sas-lib + @solana/spl-memo are devDependencies in tools/package.json)

import { getCreateAttestationInstructionDataEncoder } from 'sas-lib';
import { createMemoInstruction } from '@solana/spl-memo';

// Fixed test vector (matches tests/borsh.rs).
const NONCE   = 'MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr'; // a known pubkey used as the attestation nonce
const DATA    = Buffer.from('palinurus:temp=24.7C');          // 20-byte schema-encoded payload
const EXPIRY  = 1753000000n;                                   // i64 unix timestamp

const enc = getCreateAttestationInstructionDataEncoder();
const bytes = Buffer.from(enc.encode({ nonce: NONCE, data: DATA, expiry: EXPIRY }));

console.log('=== SAS create_attestation ix DATA (Borsh) ===');
console.log('hex   :', bytes.toString('hex'));
console.log('len   :', bytes.length);
console.log('disc  :', bytes[0], '(expect 6)');
console.log('nonce :', bytes.slice(1, 33).toString('hex'));
console.log('u32len:', bytes.readUInt32LE(33), '(expect', DATA.length, ')');
console.log('data  :', bytes.slice(37, 37 + DATA.length).toString('utf8'));
console.log('expiry:', bytes.readBigInt64LE(37 + DATA.length), '(expect', EXPIRY, ')');

console.log('\n=== Memo v3 ix DATA (raw UTF-8, no discriminator) ===');
const memoIx = createMemoInstruction(Buffer.from('sensor reading ok'), null);
console.log('hex   :', Buffer.from(memoIx.data).toString('hex'));
console.log('utf8  :', Buffer.from(memoIx.data).toString('utf8'));
console.log('prog  :', memoIx.programId.toBase58());