# SettlementOracle ZK

Parametric insurance settlement platform on Solana. When an oracle attests a real-world event (for example, rainfall below a threshold), a smart contract automatically releases escrowed funds. A zero-knowledge (ZK) proof attests that the business rule was executed correctly against the oracle data.

**Target users:** Insurers — actuaries, risk managers, pricing analysts, and auditors  
**Stage:** MVP (structure scaffold only — packages are not implemented yet)

## Architecture

```
Oracle (Pyth) → oracle-connector → Trigger evaluation
                     ↓
              programs/escrow (Anchor)
                     ↓
                 zk-prover
                     ↓
              api → web dashboard

         shared/  (payload & ID contracts across packages)
```

Full component map, trust boundaries, and runtime flow: [`docs/architecture/mvp-system-overview.md`](docs/architecture/mvp-system-overview.md).

## Packages

| Path | Purpose | Planned MVP tech |
|------|---------|------------------|
| [`programs/escrow/`](programs/escrow/) | Premium escrow and automated payout on-chain | Solana + Anchor (Rust) |
| [`oracle-connector/`](oracle-connector/) | Consume oracle feeds; staleness and confidence checks | TypeScript + Pyth Network |
| [`zk-prover/`](zk-prover/) | Off-chain ZK proof that the trigger rule ran correctly | Light Protocol or Succinct |
| [`api/`](api/) | Secure gateway for insurers: state queries and PRD payload | Rust + PostgreSQL |
| [`web/`](web/) | Settlement dashboard and proof explorer | Next.js + Solana Wallet Adapter |
| [`shared/`](shared/) | Cross-package schemas and type contracts (PRD payload, IDs) | JSON Schema / OpenAPI → Rust + TS |

## MVP scope

### In scope

- Single oracle feed via Pyth Network (climate **or** price for MVP)
- Escrow + automated payout Anchor programs on Solana devnet
- Basic ZK proof attesting trigger execution
- Web dashboard for active policies and settlement status
- API gateway returning the PRD-defined risk/proof payload

### Out of scope (not without explicit approval)

- Multi-oracle aggregation or dispute/challenge periods
- LLMs inside ZK circuits
- Mainnet deployment
- Continuous monitoring/alerting infrastructure
- Direct integration with third-party insurer core systems

## Stack (MVP)

| Component | Technology |
|-----------|------------|
| Smart contracts | Solana + Anchor (Rust) |
| Oracle | Pyth Network |
| ZK infrastructure | Light Protocol or Succinct |
| API | Rust + PostgreSQL |
| Frontend | Next.js + Solana Wallet Adapter |
| Infra | AWS (EC2/Lambda), Solana devnet RPC |

## Getting started

This repository is a **structure-only monorepo**. Package folders exist with documentation only; toolchains and application code will be added in later phases.

1. Read the product requirements: [`docs/PRD.md`](docs/PRD.md)
2. Read the MVP architecture: [`docs/architecture/mvp-system-overview.md`](docs/architecture/mvp-system-overview.md)
3. Read agent and scope conventions: [`AGENTS.md`](AGENTS.md)
4. Review shared payload contracts: [`shared/README.md`](shared/README.md)
5. Implement package-by-package following the roadmap in the PRD (Solana env → Anchor escrow → Pyth connector → ZK prover → API → web dashboard)

## Documentation

| Document | Description |
|----------|-------------|
| [`docs/PRD.md`](docs/PRD.md) | Product requirements |
| [`docs/architecture/mvp-system-overview.md`](docs/architecture/mvp-system-overview.md) | MVP system architecture |
| [`shared/README.md`](shared/README.md) | Shared contracts (payload and IDs) |
| [`AGENTS.md`](AGENTS.md) | MVP guardrails, layer map, and agent conventions |

## License

Proprietary — all rights reserved unless otherwise stated.
