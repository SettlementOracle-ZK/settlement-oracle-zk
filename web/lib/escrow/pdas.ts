import { PublicKey } from '@solana/web3.js';

const POLICY_SEED = Buffer.from('policy');
const ESCROW_SEED = Buffer.from('escrow');

export function policyPda(programId: PublicKey, policyId: Uint8Array): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync([POLICY_SEED, Buffer.from(policyId)], programId);
  return pda;
}

export function escrowPda(programId: PublicKey, policyId: Uint8Array): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync([ESCROW_SEED, Buffer.from(policyId)], programId);
  return pda;
}

export function randomPolicyId(): Uint8Array {
  const id = new Uint8Array(32);
  crypto.getRandomValues(id);
  return id;
}

export function policyIdHex(policyId: Uint8Array): string {
  return Buffer.from(policyId).toString('hex');
}
