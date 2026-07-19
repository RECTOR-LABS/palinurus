#!/usr/bin/env node
/**
 * Palinurus SAS setup — one-time Credential + Schema creation on Solana devnet.
 *
 * Creates a Credential (issuer) + a Schema (sensor-reading layout) for the
 * depin-attest plugin, then prints the PDAs for plugin config.
 *
 * Usage:
 *   RPC_ENDPOINT=https://devnet.helius.com/ \
 *   PAYER_KEYPAIR=~/.config/solana/id.json \
 *   node tools/sas-setup/sas-setup.mjs
 *
 * Requires: sas-lib, @solana/web3.js (in tools/package.json devDeps/deps).
 */

import { readFileSync } from 'fs';
import { homedir } from 'os';
import { join, resolve } from 'path';
import {
  Connection, Keypair, PublicKey, Transaction, TransactionInstruction,
  sendAndConfirmTransaction, LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import {
  deriveCredentialPda, deriveSchemaPda,
  getCreateCredentialInstruction, getCreateSchemaInstruction,
} from 'sas-lib';

// ── Config ──
const RPC_ENDPOINT = process.env.RPC_ENDPOINT || 'https://api.devnet.solana.com';
const PAYER_KEYPAIR_PATH = process.env.PAYER_KEYPAIR
  ? resolve(process.env.PAYER_KEYPAIR)
  : join(homedir(), '.config/solana/id.json');
const CREDENTIAL_NAME = process.env.CREDENTIAL_NAME || 'palinurus-depin';
const SCHEMA_NAME = process.env.SCHEMA_NAME || 'palinurus-sensor-reading-v1';
const SCHEMA_VERSION = parseInt(process.env.SCHEMA_VERSION || '1', 10);

// ── Load payer ──
const payerSecret = JSON.parse(readFileSync(PAYER_KEYPAIR_PATH, 'utf-8'));
const payer = Keypair.fromSecretKey(Uint8Array.from(payerSecret));

const conn = new Connection(RPC_ENDPOINT, 'confirmed');

// ── Helper: convert a sas-lib Instruction to a web3.js TransactionInstruction ──
function toTxInstruction(ix) {
  return new TransactionInstruction({
    programId: new PublicKey(ix.programAddress),
    keys: ix.accounts.map(a => ({
      pubkey: new PublicKey(a.address),
      isSigner: a.isSigner,
      isWritable: a.isWritable,
    })),
    data: Buffer.from(ix.data),
  });
}

async function main() {
  console.log('=== Palinurus SAS Setup ===');
  console.log(`RPC: ${RPC_ENDPOINT}`);
  console.log(`Payer: ${payer.publicKey.toBase58()}`);
  console.log();

  // Check balance
  const balance = await conn.getBalance(payer.publicKey);
  console.log(`Balance: ${balance / LAMPORTS_PER_SOL} SOL`);
  if (balance < 0.05 * LAMPORTS_PER_SOL) {
    console.warn('⚠️  Low balance — need ~0.05 SOL for Credential + Schema creation');
    console.warn('   Fund with: solana airdrop 2', payer.publicKey.toBase58());
  }
  console.log();

  // ── 1. Derive Credential PDA ──
  const [credentialPda] = await deriveCredentialPda({
    authority: payer.publicKey.toBase58(),
    name: CREDENTIAL_NAME,
  });
  console.log(`Credential PDA: ${credentialPda}`);

  // ── 2. Create Credential ──
  console.log('Creating Credential…');
  const createCredentialIx = toTxInstruction(
    getCreateCredentialInstruction({
      payer: payer.publicKey.toBase58(),
      credential: credentialPda,
      authority: payer.publicKey.toBase58(),
      name: CREDENTIAL_NAME,
      signers: [payer.publicKey.toBase58()],
    })
  );
  const credTx = new Transaction().add(createCredentialIx);
  const credSig = await sendAndConfirmTransaction(conn, credTx, [payer]);
  console.log(`✓ Credential created: ${credSig}`);
  console.log();

  // ── 3. Derive Schema PDA ──
  const [schemaPda] = await deriveSchemaPda({
    credential: credentialPda,
    name: SCHEMA_NAME,
    version: SCHEMA_VERSION,
  });
  console.log(`Schema PDA: ${schemaPda}`);

  // ── 4. Create Schema ──
  // The layout is opaque bytes stored on-chain; off-chain consumers use it to
  // decode the attestation `data` payload. We store a JSON description.
  const layout = Buffer.from(JSON.stringify({
    type: "struct",
    fields: [
    { name: "sensor_id", type: "string" },
    { name: "value", type: "f64" },
    { name: "unit", type: "string" },
    { name: "timestamp", type: "i64" },
    ],
  }));
  const fieldNames = ["sensor_id", "value", "unit", "timestamp"];

  console.log('Creating Schema…');
  const createSchemaIx = toTxInstruction(
    getCreateSchemaInstruction({
      payer: payer.publicKey.toBase58(),
      authority: payer.publicKey.toBase58(),
      credential: credentialPda,
      schema: schemaPda,
      name: SCHEMA_NAME,
      description: 'DePIN sensor reading attestation (Palinurus)',
      layout,
      fieldNames,
    })
  );
  const schemaTx = new Transaction().add(createSchemaIx);
  const schemaSig = await sendAndConfirmTransaction(conn, schemaTx, [payer]);
  console.log(`✓ Schema created: ${schemaSig}`);
  console.log();

  // ── 5. Print config block ──
  console.log('=== Setup Complete ===');
  console.log();
  console.log('Paste into config.toml:');
  console.log();
  console.log('[plugins.entries.depin_attest]');
  console.log(`rpc_endpoint = "${RPC_ENDPOINT}"`);
  console.log(`credential_pda = "${credentialPda}"`);
  console.log(`schema_pda = "${schemaPda}"`);
  console.log(`authority = "${payer.publicKey.toBase58()}"`);
  console.log(`payer = "${payer.publicKey.toBase58()}"`);
  console.log('# Create a durable nonce account separately:');
  console.log('#   solana create-nonce-account <NONCE_ACCOUNT_KEYPAIR> 0.01');
  console.log('# Then set:');
  console.log('# nonce_account = "<base58>"');
  console.log('# nonce_authority = "<base58>"');
  console.log('custody_mode = "t1"');
  console.log();

  // Also create a nonce account if requested
  if (process.env.CREATE_NONCE === 'true') {
    console.log('Creating durable nonce account…');
    const nonceKeypair = Keypair.generate();
    const nonceAuth = payer.publicKey;
    const createNonceIx = SystemProgram.createNonceAccount({
      fromPubkey: payer.publicKey,
      noncePubkey: nonceKeypair.publicKey,
      authorizedPubkey: nonceAuth,
      lamports: 0.01 * LAMPORTS_PER_SOL,
    });
    const nonceTx = new Transaction().add(createNonceIx);
    const nonceSig = await sendAndConfirmTransaction(conn, nonceTx, [payer, nonceKeypair]);
    console.log(`✓ Nonce account: ${nonceKeypair.publicKey.toBase58()}`);
    console.log(`  sig: ${nonceSig}`);
    console.log(`  authority: ${nonceAuth.toBase58()}`);
    console.log(`# nonce_account = "${nonceKeypair.publicKey.toBase58()}"`);
    console.log(`# nonce_authority = "${nonceAuth.toBase58()}"`);
  }
}

// Need SystemProgram import for nonce creation
import { SystemProgram } from '@solana/web3.js';

main().catch(e => {
  console.error('✗ Setup failed:', e);
  process.exit(1);
});
