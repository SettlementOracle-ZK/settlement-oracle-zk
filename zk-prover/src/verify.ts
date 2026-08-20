import { verifyWitnessHash } from './commitment.js';
import type { CircuitWitness, VerifyResult } from './types.js';

export function verifyProof(witness: CircuitWitness, proofHash: string): VerifyResult {
  return {
    verified: verifyWitnessHash(witness, proofHash),
    proof_hash: proofHash,
    witness,
  };
}
