export { buildWitness, computeModelConfidence, computeRiskScore, evaluateCircuit } from './circuit.js';
export type { EvaluateInputs } from './circuit.js';
export { canonicalWitnessJson, hashWitness, verifyWitnessHash } from './commitment.js';
export { buildPrdPayload, generateProof } from './prove.js';
export type { GenerateProofOptions } from './prove.js';
export type {
  CircuitWitness,
  PrdSettlementPayload,
  ProofArtifact,
  TriggerOperator,
  VerifyResult,
} from './types.js';
export { verifyProof } from './verify.js';
