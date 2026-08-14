# API Gateway

Secure HTTP gateway for insurers to query settlement state. On-chain escrow is the source of truth for balances and status; PostgreSQL is an index/cache.

## Status

Phase 2 scaffold: `GET /health`, `GET /policies/:id` (hex `policy_id`), schema `policies` + `settlements`.

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Process + Postgres connectivity |
| GET | `/policies/:id` | Policy + escrow status from Solana RPC (`:id` = 32-byte hex) |

`GET /settlements/:id` and `GET /verify/:proofHash` are later phases.

### `GET /policies/:id` response

```json
{
  "policy_id": "<hex>",
  "holder": "<pubkey>",
  "expiry": 4102444800,
  "asset_class": "agriculture_climate",
  "escrow": {
    "status": "Active",
    "amount": 500000000,
    "trigger_threshold": 100000000000,
    "paused": false,
    "authority": "<pubkey>"
  },
  "pdas": { "policy": "<pubkey>", "escrow": "<pubkey>" }
}
```

404 if the on-chain policy PDA is missing.

## Run locally

```bash
# from repo root
cp .env.example .env
docker compose up -d postgres
# Postgres is published on host port 5433 (avoids clashing with a local 5432)
cargo run --manifest-path api/Cargo.toml
```

This crate is **not** a member of the root Anchor workspace (avoids Solana crate conflicts). Build it with `--manifest-path api/Cargo.toml`.

Env vars: see [`.env.example`](../.env.example).

## Related

- Product requirements: [`../docs/PRD.md`](../docs/PRD.md)
- Architecture: [`../docs/architecture/mvp-system-overview.md`](../docs/architecture/mvp-system-overview.md)
- Escrow program: [`../programs/escrow/`](../programs/escrow/)
