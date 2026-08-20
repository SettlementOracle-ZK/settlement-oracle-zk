import { describe, expect, it } from 'vitest';

import {
  buildWitness,
  canonicalWitnessJson,
  generateProof,
  hashWitness,
  verifyProof,
} from '../src/index.js';

const BASE = {
  feedId: '0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d',
  oraclePrice: 87.2,
  oracleConf: 0.5,
  publishTime: 1_700_000_000,
  threshold: 100,
  operator: 'lt' as const,
  assetClass: 'agriculture_climate',
  nowSeconds: 1_700_000_030,
  verificationBaseUrl: 'http://127.0.0.1:3000',
  timestamp: '2026-05-19T14:42:00Z',
};

describe('generateProof', () => {
  it('is deterministic for identical inputs', () => {
    const a = generateProof(BASE);
    const b = generateProof(BASE);
    expect(a.proof_hash).toBe(b.proof_hash);
    expect(a.payload.risk_score).toBe(b.payload.risk_score);
  });

  it('sets triggered when price below threshold', () => {
    const proof = generateProof(BASE);
    expect(proof.witness.triggered).toBe(true);
    expect(proof.payload.zk_proof.hash).toMatch(/^0x[0-9a-f]{64}$/);
    expect(proof.payload.zk_proof.verification_url).toContain(proof.proof_hash);
  });

  it('verifyProof accepts matching hash', () => {
    const proof = generateProof(BASE);
    const result = verifyProof(proof.witness, proof.proof_hash);
    expect(result.verified).toBe(true);
  });

  it('canonical JSON is stable', () => {
    const witness = buildWitness(BASE);
    expect(canonicalWitnessJson(witness)).toBe(canonicalWitnessJson(witness));
    expect(hashWitness(witness)).toMatch(/^0x[0-9a-f]{64}$/);
  });
});
