CREATE TABLE IF NOT EXISTS policies (
    policy_id BYTEA PRIMARY KEY CHECK (octet_length(policy_id) = 32),
    holder TEXT NOT NULL,
    expiry TIMESTAMPTZ NOT NULL,
    asset_class TEXT NOT NULL,
    policy_pda TEXT NOT NULL,
    escrow_pda TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS settlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id BYTEA NOT NULL REFERENCES policies (policy_id),
    status TEXT NOT NULL,
    payout_amount BIGINT,
    tx_signature TEXT,
    proof_hash TEXT,
    settled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS settlements_policy_id_idx ON settlements (policy_id);
