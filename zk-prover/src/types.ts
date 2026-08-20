export type TriggerOperator = 'lt' | 'lte' | 'gt' | 'gte';

export interface CircuitWitness {
  feed_id: string;
  oracle_price: number;
  oracle_conf: number;
  publish_time: number;
  threshold: number;
  operator: TriggerOperator;
  triggered: boolean;
  risk_score: number;
  asset_class: string;
}

export interface PrdSettlementPayload {
  asset_class: string;
  risk_score: number;
  scale: '0-100';
  model_confidence: string;
  timestamp: string;
  zk_proof: {
    hash: string;
    verification_url: string;
  };
}

export interface ProofArtifact {
  witness: CircuitWitness;
  proof_hash: string;
  payload: PrdSettlementPayload;
}

export interface VerifyResult {
  verified: boolean;
  proof_hash: string;
  witness: CircuitWitness;
}
