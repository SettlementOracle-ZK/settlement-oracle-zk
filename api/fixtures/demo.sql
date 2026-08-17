-- Local demo index rows for the dashboard (Explorer + /verify).
-- Applied only by `migrate seed` / `make db-seed`, never by schema migrations.
-- Settlements are TRIGGERED without a payout signature — PAID requires an on-chain tx.

INSERT INTO policies (policy_id, holder, expiry, asset_class, policy_pda, escrow_pda)
VALUES (
    decode('aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'hex'),
    'DemoHolder111111111111111111111111111111111',
    '2099-12-31T00:00:00Z',
    'agriculture_climate',
    'Policy1111111111111111111111111111111111111',
    'Escrow1111111111111111111111111111111111111'
)
ON CONFLICT (policy_id) DO NOTHING;

INSERT INTO proofs (
    proof_hash,
    asset_class,
    risk_score,
    scale,
    model_confidence,
    proof_timestamp,
    public_inputs
) VALUES (
    '0xabc123def4567890abc123def4567890abc123def4567890abc123def4567890',
    'agriculture_climate',
    85.4,
    '0-100',
    '92%',
    '2026-05-19T14:42:00Z',
    '{"triggered": true, "threshold": 100, "oracle_price": 87.2, "operator": "lt"}'::jsonb
)
ON CONFLICT (proof_hash) DO NOTHING;

INSERT INTO settlements (
    id,
    policy_id,
    status,
    payout_amount,
    tx_signature,
    proof_hash,
    settled_at
) VALUES (
    '11111111-1111-4111-8111-111111111111',
    decode('aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'hex'),
    'TRIGGERED',
    NULL,
    NULL,
    '0xabc123def4567890abc123def4567890abc123def4567890abc123def4567890',
    NULL
)
ON CONFLICT (id) DO NOTHING;
