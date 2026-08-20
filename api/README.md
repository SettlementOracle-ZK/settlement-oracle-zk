# API Gateway

Secure HTTP gateway for insurers to query settlement state. On-chain escrow is the source of truth for balances and status; PostgreSQL is an index/cache.

## Status

Phase 4: `GET /settlements/:id` (full PRD payload), dev-only `POST /proofs` and `POST /settlements/register` for E2E indexing.

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Process + Postgres connectivity |
| GET | `/policies` | Indexed policy list (DB cache; local demo rows via `make db-seed`) |
| GET | `/policies/:id` | Policy + escrow status from Solana RPC (`:id` = 32-byte hex) |
| GET | `/settlements` | Indexed settlements (tx signature, proof hash, verify URL) |
| GET | `/settlements/:id` | Settlement + full PRD payload (joins `proofs`) |
| GET | `/verify/:proofHash` | ZK attestation; `circuit_commitment` when witness matches hash |
| GET | `/oracle/latest` | Pyth Hermes SOL/USD tick + staleness / confidence flags |
| POST | `/proofs` | **Dev only** — register proof witness + PRD fields |
| POST | `/settlements/register` | **Dev only** — index settlement (+ optional policy upsert) |

404 if the on-chain policy PDA is missing (`GET /policies/:id`) or the proof hash is unknown (`GET /verify/:proofHash`).

### `GET /verify/:proofHash` response (PRD + additive fields)

```json
{
  "asset_class": "agriculture_climate",
  "risk_score": 85.4,
  "scale": "0-100",
  "model_confidence": "92%",
  "timestamp": "2026-05-19T14:42:00Z",
  "zk_proof": {
    "hash": "0xabc123...",
    "verification_url": "http://127.0.0.1:3000/verify/0xabc123..."
  },
  "verified": false,
  "attested": true,
  "verification_method": "stored_attestation | circuit_commitment",
  "public_inputs": { "triggered": true }
}
```

MVP verification: **stored attestation** plus optional **circuit_commitment** when `public_inputs` recomputes to `proof_hash`.

## Run locally

```bash
# from repo root
cp .env.example .env
make db-up          # Postgres on :5433
make db-migrate     # sqlx schema migrations
make db-seed        # optional local demo index rows
cargo run --manifest-path api/Cargo.toml
```

`make db-migrate` is optional if you boot the API — `cargo run` applies pending **schema** migrations on startup. Demo rows are never auto-applied.

If `make db-migrate` errors because a removed seed migration is still recorded locally, run `make db-reset` (requires `APP_ENV=development` in `.env`).

```bash
make db-status
cargo run --manifest-path api/Cargo.toml --bin migrate -- add add_settlement_index
make db-reset       # local/dev only; drops public schema
```

This crate is **not** a member of the root Anchor workspace (avoids Solana crate conflicts). Build it with `--manifest-path api/Cargo.toml`.

Postgres is published on host port **5433**. Env vars: [`.env.example`](../.env.example).

## Tests

```bash
make db-up
DATABASE_URL=postgres://settlement:settlement@127.0.0.1:5433/settlement \
  cargo test --manifest-path api/Cargo.toml
```

## Related

- Product requirements: [`../docs/PRD.md`](../docs/PRD.md)
- Architecture: [`../docs/architecture/mvp-system-overview.md`](../docs/architecture/mvp-system-overview.md)
- Escrow program: [`../programs/escrow/`](../programs/escrow/)
