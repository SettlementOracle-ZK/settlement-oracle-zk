CREATE TABLE IF NOT EXISTS proofs (
    proof_hash TEXT PRIMARY KEY,
    asset_class TEXT NOT NULL,
    risk_score DOUBLE PRECISION NOT NULL,
    scale TEXT NOT NULL DEFAULT '0-100',
    model_confidence TEXT NOT NULL,
    proof_timestamp TIMESTAMPTZ NOT NULL,
    public_inputs JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
