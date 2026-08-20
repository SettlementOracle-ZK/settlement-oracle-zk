import { buildWitness, computeModelConfidence, type EvaluateInputs } from './circuit.js';
import { hashWitness } from './commitment.js';
import type { ProofArtifact, PrdSettlementPayload } from './types.js';

export interface GenerateProofOptions extends EvaluateInputs {
  verificationBaseUrl: string;
  timestamp?: string;
}

export function buildPrdPayload(
  witness: ReturnType<typeof buildWitness>,
  proofHash: string,
  verificationBaseUrl: string,
  timestamp: string,
): PrdSettlementPayload {
  const base = verificationBaseUrl.replace(/\/$/, '');
  return {
    asset_class: witness.asset_class,
    risk_score: witness.risk_score,
    scale: '0-100',
    model_confidence: computeModelConfidence(witness.oracle_price, witness.oracle_conf),
    timestamp,
    zk_proof: {
      hash: proofHash,
      verification_url: `${base}/verify/${proofHash}`,
    },
  };
}

export function generateProof(options: GenerateProofOptions): ProofArtifact {
  const witness = buildWitness(options);
  const proof_hash = hashWitness(witness);
  const timestamp = options.timestamp ?? new Date().toISOString();
  const payload = buildPrdPayload(witness, proof_hash, options.verificationBaseUrl, timestamp);
  return { witness, proof_hash, payload };
}
