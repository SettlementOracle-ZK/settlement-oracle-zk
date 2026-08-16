import type { OracleFeed, PolicyIndex, SettlementIndex, VerifyPayload } from './types';

export const DEMO_PROOF_HASH =
  '0xabc123def4567890abc123def4567890abc123def4567890abc123def4567890';

export const DEMO_POLICY_ID = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';

export const FIXTURE_POLICIES: PolicyIndex[] = [
  {
    policy_id: DEMO_POLICY_ID,
    holder: 'FarmCo · drought cover',
    expiry: '2099-12-31T00:00:00Z',
    asset_class: 'agriculture_climate',
    policy_pda: 'Policy1111111111111111111111111111111111111',
    escrow_pda: 'Escrow1111111111111111111111111111111111111',
  },
  {
    policy_id: 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
    holder: 'Coastal grain desk',
    expiry: '2027-06-01T00:00:00Z',
    asset_class: 'agriculture_climate',
    policy_pda: 'Policy2222222222222222222222222222222222222',
    escrow_pda: 'Escrow2222222222222222222222222222222222222',
  },
];

export const FIXTURE_SETTLEMENTS: SettlementIndex[] = [
  {
    id: '11111111-1111-4111-8111-111111111111',
    policy_id: DEMO_POLICY_ID,
    status: 'PAID',
    payout_amount: 500_000_000,
    tx_signature: '5DemoTxSignature111111111111111111111111111111111111111111111111',
    proof_hash: DEMO_PROOF_HASH,
    verification_url: `http://127.0.0.1:3000/verify/${DEMO_PROOF_HASH}`,
    settled_at: '2026-05-19T14:42:08Z',
  },
  {
    id: '22222222-2222-4222-8222-222222222222',
    policy_id: 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
    status: 'PENDING',
    payout_amount: null,
    tx_signature: null,
    proof_hash: null,
    verification_url: null,
    settled_at: null,
  },
];

export const FIXTURE_VERIFY: Record<string, VerifyPayload> = {
  [DEMO_PROOF_HASH]: {
    asset_class: 'agriculture_climate',
    risk_score: 85.4,
    scale: '0-100',
    model_confidence: '92%',
    timestamp: '2026-05-19T14:42:00Z',
    zk_proof: {
      hash: DEMO_PROOF_HASH,
      verification_url: `http://127.0.0.1:3000/verify/${DEMO_PROOF_HASH}`,
    },
    verified: true,
    verification_method: 'stored_attestation',
    public_inputs: {
      triggered: true,
      threshold: 100,
      oracle_price: 87.2,
      operator: 'lt',
    },
  },
};

export const FIXTURE_ORACLE: OracleFeed = {
  feed_id: '0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d',
  symbol: 'SOL/USD',
  price: 142.35,
  conf: 0.18,
  expo: -8,
  publish_time: Math.floor(Date.now() / 1000) - 4,
  timestamp: new Date().toISOString(),
  age_seconds: 4,
  stale: false,
  low_confidence: false,
  max_staleness_seconds: 60,
  max_confidence_ratio: 0.05,
};
