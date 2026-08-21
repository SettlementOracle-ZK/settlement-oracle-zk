/**
 * Program-owned mock Pyth PDA (seed `mock_pyth`) for devnet smoke without cloning real feeds.
 */

import { PublicKey } from "@solana/web3.js";

export const MOCK_PYTH_SEED = Buffer.from("mock_pyth");
export const MOCK_PYTH_MAGIC = 0xa1b2c3d4;

export function mockPythPda(programId: PublicKey): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync([MOCK_PYTH_SEED], programId);
  return pda;
}

export function isInitializedMockFeed(data: Buffer | null | undefined): boolean {
  if (!data || data.length < 4) {
    return false;
  }
  return data.readUInt32LE(0) === MOCK_PYTH_MAGIC;
}
