/**
 * Build a legacy-format Pyth price account JSON for solana-test-validator --account.
 * Devnet account 7UVimff... is a Pyth Receiver account (~134 B), not legacy layout.
 */

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";

const MAGIC = 0xa1b2c3d4;
const VERSION_2 = 2;
const ACCOUNT_TYPE_PRICE = 3;
const PRICE_STATUS_TRADING = 1;
const ACCOUNT_SIZE = 3312;
const AGG_OFFSET = 208;
const TIMESTAMP_OFFSET = 96;

/** Simulated flight delay in minutes (>= typical 120 min trigger). */
const DEFAULT_DELAY_MINUTES = 150n;
const DEFAULT_CONF = 1n;

function writeI64(buf: Buffer, offset: number, value: bigint): void {
  buf.writeBigInt64LE(value, offset);
}

function writeU64(buf: Buffer, offset: number, value: bigint): void {
  buf.writeBigUInt64LE(value, offset);
}

export function buildLegacyPriceData(
  price: bigint,
  conf: bigint,
  publishTime: number,
): Buffer {
  const buf = Buffer.alloc(ACCOUNT_SIZE);
  buf.writeUInt32LE(MAGIC, 0);
  buf.writeUInt32LE(VERSION_2, 4);
  buf.writeUInt32LE(ACCOUNT_TYPE_PRICE, 8);
  buf.writeInt32LE(-8, 20);
  writeI64(buf, TIMESTAMP_OFFSET, BigInt(publishTime));
  writeI64(buf, AGG_OFFSET, price);
  writeU64(buf, AGG_OFFSET + 8, conf);
  buf.writeUInt8(PRICE_STATUS_TRADING, AGG_OFFSET + 16);
  writeU64(buf, AGG_OFFSET + 24, 1n);
  return buf;
}

export function mockPythKeypair(): Keypair {
  const seed = crypto.createHash("sha256").update("settlement-oracle-mock-pyth-v1").digest();
  return Keypair.fromSeed(seed.subarray(0, 32));
}

export function writeValidatorAccountFixture(outPath: string, feed = mockPythKeypair()): string {
  const publishTime = Math.floor(Date.now() / 1000);
  const data = buildLegacyPriceData(DEFAULT_DELAY_MINUTES, DEFAULT_CONF, publishTime);
  const fixture = {
    pubkey: feed.publicKey.toBase58(),
    account: {
      lamports: LAMPORTS_PER_SOL / 10,
      data: [data.toString("base64"), "base64"],
      owner: SystemProgram.programId.toBase58(),
      executable: false,
      rentEpoch: 0,
    },
  };
  fs.mkdirSync(path.dirname(outPath), { recursive: true });
  fs.writeFileSync(outPath, JSON.stringify(fixture));
  return feed.publicKey.toBase58();
}

if (process.argv[1]?.endsWith("install-mock-pyth.ts")) {
  const out =
    process.argv[2] ??
    path.join(path.dirname(new URL(import.meta.url).pathname), ".local", "mock-pyth.json");
  const pubkey = writeValidatorAccountFixture(out);
  console.log(pubkey);
}
