# MVP System Overview

Architecture overview for **SettlementOracle ZK** at MVP scope (Solana devnet / testnet).  
Source of truth for product requirements: [`../PRD.md`](../PRD.md).  
Agent conventions and scope guard: [`../../AGENTS.md`](../../AGENTS.md).

## Goals

| Goal | MVP approach |
|------|----------------|
| Bridge real-world data to payout | Pyth Network feed → trigger evaluation |
| Hold and release premiums safely | Anchor escrow program on Solana |
| Prove rule integrity without manual audit | Off-chain ZK proof; hash reference for audit / API |
| Give insurers a query surface | Rust API + Next.js dashboard |

## Component map

```
                    ┌─────────────────────┐
                    │  Pyth Network       │
                    │  (oracle feeds)     │
                    └──────────┬──────────┘
                               │ on-chain accounts
                               ▼
┌──────────────────────────────────────────────────────────────┐
│  oracle-connector                                            │
│  Read feeds · staleness / confidence checks · trigger inputs │
└──────────────────────────────┬───────────────────────────────┘
                               │ trigger decision inputs
               ┌───────────────┴───────────────┐
               ▼                               ▼
┌──────────────────────────┐     ┌─────────────────────────────┐
│  programs/escrow         │     │  zk-prover                  │
│  Anchor (Rust)           │     │  Rule evaluation proof      │
│  Escrow vault · payout   │     │  SNARK artifact + hash      │
└────────────┬─────────────┘     └──────────────┬──────────────┘
             │ on-chain state                    │ proof hash / blob
             └──────────────┬────────────────────┘
                            ▼
               ┌────────────────────────────┐
               │  api (Rust + PostgreSQL)   │
               │  REST · PRD JSON payload   │
               └────────────┬───────────────┘
                            │
                            ▼
               ┌────────────────────────────┐
               │  web (Next.js)             │
               │  Dashboard · proof explorer│
               └────────────────────────────┘

               ┌────────────────────────────┐
               │  shared/                   │
               │  Cross-cutting contracts   │
               │  (payload, IDs, proof meta)│
               └────────────────────────────┘
```

Cross-cutting types and API/proof shapes live in [`../../shared/`](../../shared/) so Rust services and the TypeScript frontend do not invent incompatible schemas.

## Runtime flow (happy path)

1. **Policy creation** — Insurer configures threshold / strike and funds escrow via the Anchor program (devnet). Metadata may be indexed in PostgreSQL by the API.
2. **Oracle observation** — `oracle-connector` reads the chosen Pyth feed, rejects stale or low-confidence data, and emits trigger evaluation inputs.
3. **Trigger evaluation** — If the feed value satisfies the policy rule, the system is clear to settle.
4. **On-chain payout** — Escrow program transfers funds to the insured wallet when instruction + accounts authorize the payout. **Funds never move off-chain.**
5. **ZK attestation** — `zk-prover` proves that the rule was evaluated correctly over the oracle inputs (and related public parameters). MVP stores a **proof hash** for audit; full on-chain verification is post-MVP.
6. **Query** — `api` returns the PRD settlement payload (risk fields + `zk_proof`). `web` displays policies, settlements, and verification links.

## Trust boundaries

| Boundary | Trust assumption (MVP) |
|----------|------------------------|
| Pyth feed | Treated as the single authoritative data source; still subject to staleness and confidence gates |
| Anchor escrow | On-chain truth for balances and settlement authorization |
| ZK prover | Cryptographic attestation of rule execution; hash exposed via API |
| API / DB | Index and cache for UX; **not** source of truth for funds |
| Wallet users | Sign only their authorized transactions; no server-held user funds |

## Data: PRD settlement payload

Canonical JSON for insurer-facing settlement status (also defined in shared contracts):

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

MVP API surface (see cursor rule `api-gateway`):

- `GET /policies/:id`
- `GET /settlements/:id`
- `GET /verify/:proofHash`

## Package ownership

| Path | Role |
|------|------|
| [`programs/escrow/`](../../programs/escrow/) | Escrow + automated payout on Solana |
| [`oracle-connector/`](../../oracle-connector/) | Pyth consumption and validation |
| [`zk-prover/`](../../zk-prover/) | Off-chain proof generation / helpers |
| [`api/`](../../api/) | Rust HTTP gateway + PostgreSQL index |
| [`web/`](../../web/) | Settlement dashboard and proof explorer |
| [`shared/`](../../shared/) | Schema / type contracts shared across packages |

## Stack (MVP locked)

| Layer | Technology |
|-------|------------|
| Smart contracts | Solana + Anchor (Rust) |
| Oracle | Pyth Network (single feed class for MVP) |
| ZK | Light Protocol or Succinct (chosen at implementation) |
| API | Rust + PostgreSQL |
| Frontend | Next.js + Solana Wallet Adapter |
| Network | Solana devnet / testnet only |

## MVP in / out (architecture impact)

**In:** one oracle feed path, escrow + payout on devnet, basic ZK trigger proof, monitoring dashboard, PRD-shaped API responses.

**Out:** multi-oracle aggregation, dispute/challenge windows, LLMs in circuits, mainnet, continuous monitoring product, direct writes into third-party insurer cores.

## Deployed program (devnet)

| Field | Value |
|-------|--------|
| Program | `escrow` |
| Program ID | `987M3ZdtXNuZu7jfA1TtTHNgYThNHEYyGVP5sq42j1Rd` |
| Cluster | Solana devnet (MVP) |
| Pyth SOL/USD (legacy) | `7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE` |

Configure via `ESCROW_PROGRAM_ID`, `PYTH_PRICE_FEED`, `SOLANA_RPC_URL` in [`.env.example`](../../.env.example).

## Open decisions (document when resolved)

1. ~~**Feed class for MVP**~~ — **price feed** (SOL/USD) chosen; climate swap is post-MVP.
2. **ZK stack** — MVP uses SHA-256 commitment; Light Protocol vs Succinct for production SNARK.
3. ~~**HTTP framework** for `api`~~ — **Axum** (in use).
4. **Shared package distribution** — JSON Schema / OpenAPI only first, then Rust crate and TS types generated from the same source.

## Related documents

- [`../PRD.md`](../PRD.md) — product requirements and roadmap
- [`../../AGENTS.md`](../../AGENTS.md) — layer map and agent behavior
- [`../../README.md`](../../README.md) — monorepo entry point
- Package READMEs under each top-level package folder
