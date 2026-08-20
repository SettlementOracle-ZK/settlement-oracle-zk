# SettlementOracle ZK

Parametric insurance settlement platform on Solana. When an oracle attests a real-world event (for example, rainfall below a threshold), a smart contract automatically releases escrowed funds. A zero-knowledge (ZK) proof attests that the business rule was executed correctly against the oracle data.

**Target users:** Insurers — actuaries, risk managers, pricing analysts, and auditors  
**Stage:** MVP — devnet-ready; local stack documented in [`docs/runbooks/run-locally.md`](docs/runbooks/run-locally.md)

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
| [`zk-prover/`](zk-prover/) | Off-chain ZK proof that the trigger rule ran correctly | TypeScript commitment prover |
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

Full local setup: [`docs/runbooks/run-locally.md`](docs/runbooks/run-locally.md)

**Prerequisites:** Docker, Solana CLI, Anchor 1.1.x, Rust (see [`rust-toolchain.toml`](rust-toolchain.toml)), Node 20+.

```bash
cp .env.example .env
make db-up && make db-migrate
cargo run --manifest-path api/Cargo.toml          # API :3000
cd web && npm install && npm run dev              # dashboard :3001
make test-all                                     # escrow + oracle + zk + api
make settlement-flow                              # oracle → prover → API index
```

On-chain devnet: [`docs/runbooks/devnet-smoke.md`](docs/runbooks/devnet-smoke.md)

```bash
PATH="$HOME/.cargo/bin:$PATH" anchor build --ignore-keys
PATH="$HOME/.cargo/bin:$PATH" anchor test
```

## Documentation

| Document | Description |
|----------|-------------|
| [`docs/PRD.md`](docs/PRD.md) | Product requirements |
| [`docs/architecture/mvp-system-overview.md`](docs/architecture/mvp-system-overview.md) | MVP system architecture |
| [`shared/README.md`](shared/README.md) | Shared contracts (payload and IDs) |
| [`docs/runbooks/run-locally.md`](docs/runbooks/run-locally.md) | Run full stack locally |
| [`docs/runbooks/devnet-smoke.md`](docs/runbooks/devnet-smoke.md) | Devnet deploy + smoke |
| [`docs/checklists/mvp-prd-checklist.md`](docs/checklists/mvp-prd-checklist.md) | MVP vs PRD (Phase 5) |

## License

Proprietary — all rights reserved unless otherwise stated.
