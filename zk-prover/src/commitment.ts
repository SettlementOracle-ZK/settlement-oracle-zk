import { createHash } from 'node:crypto';

import type { CircuitWitness } from './types.js';

/** Stable JSON for cross-language verify (API Rust mirrors this). */
export function canonicalWitnessJson(witness: CircuitWitness): string {
  const ordered: Record<string, unknown> = {
    asset_class: witness.asset_class,
    feed_id: witness.feed_id,
    operator: witness.operator,
    oracle_conf: witness.oracle_conf,
    oracle_price: witness.oracle_price,
    publish_time: witness.publish_time,
    risk_score: witness.risk_score,
    threshold: witness.threshold,
    triggered: witness.triggered,
  };
  return JSON.stringify(ordered);
}

export function hashWitness(witness: CircuitWitness): string {
  const digest = createHash('sha256').update(canonicalWitnessJson(witness)).digest('hex');
  return `0x${digest}`;
}

export function verifyWitnessHash(witness: CircuitWitness, expectedHash: string): boolean {
  const normalized = expectedHash.startsWith('0x') ? expectedHash.toLowerCase() : `0x${expectedHash.toLowerCase()}`;
  return hashWitness(witness) === normalized;
}
