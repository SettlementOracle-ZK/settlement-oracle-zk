# API Gateway

Secure HTTP gateway for insurers to query settlement state. On-chain escrow is the source of truth for balances and status; PostgreSQL is an index/cache.

## Status

Phase 3: `GET /health`, `GET /policies/:id`, `GET /policies`, `GET /settlements`, `GET /verify/:proofHash`, `GET /oracle/latest`.

`GET /settlements/:id` with the full PRD payload remains Fase 4 (task 4.8).

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Process + Postgres connectivity |
| GET | `/policies` | Indexed policy list (DB cache; empty until seeded) |
| GET | `/policies/:id` | Policy + escrow status from Solana RPC (`:id` = 32-byte hex) |
| GET | `/settlements` | Indexed settlements (tx signature, proof hash, verify URL) |
| GET | `/verify/:proofHash` | Off-chain ZK attestation lookup (PRD payload) |
| GET | `/oracle/latest` | Pyth Hermes SOL/USD tick + staleness / confidence flags |

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
  "verified": true,
  "verification_method": "stored_attestation",
  "public_inputs": { "triggered": true }
}
```

MVP verification is an **indexed attestation**: the hash is stored when a proof is produced. Circuit-native verify lands with Rodrigo 3.1–3.3; this endpoint stays the same shape.

## Run locally

```bash
# from repo root
cp .env.example .env
docker compose up -d postgres
psql "$DATABASE_URL" -f api/fixtures/demo.sql
cargo run --manifest-path api/Cargo.toml
```

This crate is **not** a member of the root Anchor workspace (avoids Solana crate conflicts). Build it with `--manifest-path api/Cargo.toml`.

Postgres is published on host port **5433**. Env vars: [`.env.example`](../.env.example).

## Tests

```bash
docker compose up -d postgres
DATABASE_URL=postgres://settlement:settlement@127.0.0.1:5433/settlement \
  cargo test --manifest-path api/Cargo.toml
```

## Related

- Product requirements: [`../docs/PRD.md`](../docs/PRD.md)
- Architecture: [`../docs/architecture/mvp-system-overview.md`](../docs/architecture/mvp-system-overview.md)
- Escrow program: [`../programs/escrow/`](../programs/escrow/)
