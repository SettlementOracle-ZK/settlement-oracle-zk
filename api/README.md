# API Gateway

Secure HTTP gateway for insurers to query settlement state, risk outputs, and ZK proof metadata.

## Status

**Scaffold only — not implemented**

## Purpose

- Expose authenticated REST (or similar) endpoints for insurers
- Aggregate program account state, oracle-backed metrics, and ZK artifacts
- Return the PRD-defined JSON payload for risk / proof inspection

## Example response shape (PRD)

```json
{
  "asset_class": "agriculture_climate",
  "risk_score": 85.4,
  "scale": "0-100",
  "model_confidence": "92%",
  "timestamp": "2026-05-19T14:42:00Z",
  "zk_proof": {
    "hash": "0xABC123...",
    "verification_url": "https://api.riskoracle.com/verify/0xABC..."
  }
}
```

## Responsibilities

| Area | Description |
|------|-------------|
| Gateway | Auth, validation, rate limiting (MVP-appropriate level) |
| Solana RPC | Read program accounts and settlement status |
| Proof surface | Serve verification URLs / hashes from `zk-prover` |
| Persistence | Policy and audit records in PostgreSQL |

## Target stack

- **Language / runtime:** Rust
- **HTTP framework (examples):** Axum or Actix Web (chosen at implementation time)
- **Database:** PostgreSQL
- **Solana client:** Rust Solana SDK / RPC client crates
- **RPC:** Solana devnet for MVP
- **Infra (later):** AWS EC2 / Lambda as needed

## Future layout (when implemented)

```
api/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── routes/
│   ├── services/
│   ├── db/
│   └── error.rs
└── tests/
```

## Non-goals (MVP)

- Acting as a broker that mutates third-party insurer core systems
- Multi-chain L2 routers beyond Solana MVP path
- Continuous monitoring fans-out as a product surface
- Node.js / TypeScript implementation (this service is Rust)

## Related

- Product requirements: [`../docs/PRD.md`](../docs/PRD.md)
- Architecture: [`../docs/architecture/mvp-system-overview.md`](../docs/architecture/mvp-system-overview.md)
- Shared contracts: [`../shared/README.md`](../shared/README.md)
- Agent conventions: [`../AGENTS.md`](../AGENTS.md)
- Cursor rule: `api-gateway`
- Upstream: [`../programs/escrow/`](../programs/escrow/), [`../oracle-connector/`](../oracle-connector/), [`../zk-prover/`](../zk-prover/)
- UI consumer: [`../web/`](../web/)
