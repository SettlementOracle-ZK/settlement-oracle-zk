export type SettlementStatus = 'PENDING' | 'TRIGGERED' | 'PAID' | 'FAILED';

export interface PolicyIndex {
  policy_id: string;
  holder: string;
  expiry: string;
  asset_class: string;
  policy_pda: string;
  escrow_pda: string;
}

export interface PolicyDetail {
  policy_id: string;
  holder: string;
  expiry: number;
  asset_class: string;
  escrow: {
    status: string;
    amount: number;
    trigger_threshold: number;
    paused: boolean;
    authority: string;
  };
  pdas: {
    policy: string;
    escrow: string;
  };
}

export interface SettlementIndex {
  id: string;
  policy_id: string;
  status: string;
  payout_amount: number | null;
  tx_signature: string | null;
  proof_hash: string | null;
  verification_url: string | null;
  settled_at: string | null;
}

export interface VerifyPayload {
  asset_class: string;
  risk_score: number;
  scale: string;
  model_confidence: string;
  timestamp: string;
  zk_proof: {
    hash: string;
    verification_url: string;
  };
  attested: boolean;
  verified: boolean;
  verification_method: string;
  public_inputs: Record<string, unknown>;
}

export interface OracleFeed {
  feed_id: string;
  symbol: string;
  price: number;
  conf: number;
  expo: number;
  publish_time: number;
  timestamp: string;
  age_seconds: number;
  stale: boolean;
  low_confidence: boolean;
  max_staleness_seconds: number;
  max_confidence_ratio: number;
}
