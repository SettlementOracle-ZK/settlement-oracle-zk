# SettlementOracle ZK — Agent Guide

## Overview

SettlementOracle ZK is a parametric insurance settlement platform on Solana.
When an oracle attests a real-world event (e.g., rainfall below threshold), a
smart contract automatically releases escrowed funds. A ZK proof attests that
the business rule was executed correctly against the oracle data.

**Users:** Insurers (actuaries, risk managers, pricing analysts, auditors)
**Stage:** MVP — devnet/testnet only

## MVP Scope

### IN
- Single oracle feed via Pyth Network (climate OR price — pick one for MVP)
- Escrow + automated payout Anchor programs on Solana devnet
- Basic ZK proof attesting trigger execution
- Web dashboard for monitoring active policies and settlement status
- API Gateway returning PRD-defined payload schema

### OUT (do not implement without explicit approval)
- Multi-oracle aggregation or dispute/challenge periods
- LLMs inside ZK circuits
- Mainnet deployment
- Continuous monitoring/alerting infrastructure
- Direct integration with third-party insurer core systems

## Architecture

```
Oracle (Pyth) → Oracle Connector → Trigger Evaluation
                                        ↓
                              Anchor Program (Escrow/Payout)
                                        ↓
                              ZK Prover (off-chain proof)
                                        ↓
                              API Gateway → Dashboard
```

## Layer Map

| Layer           | Path               | Rule                        | Skill                      |
|-----------------|--------------------|-----------------------------|----------------------------|
| Smart Contracts | `programs/`        | `anchor-programs.mdc`       | `anchor-program-dev`       |
| Oracle          | `oracle-connector/`| `oracle-integration.mdc`    | `pyth-oracle-integration`  |
| ZK Proofs       | `zk-prover/`       | `zk-proofs.mdc`             | `zk-proof-generation`      |
| API             | `api/`             | `api-gateway.mdc`           | —                          |
| Frontend        | `web/`             | `frontend-dashboard.mdc`    | —                          |
| Shared contracts| `shared/`          | —                           | —                          |
| Domain (cross)  | —                  | `project-core.mdc`          | `settlement-oracle-domain` |

## Stack (locked for MVP)

| Component        | Technology                              |
|------------------|-----------------------------------------|
| Smart Contracts  | Solana + Anchor (Rust)                  |
| Oracle           | Pyth Network                            |
| ZK Infrastructure| Light Protocol or Succinct              |
| API              | Rust + PostgreSQL                       |
| Frontend         | Next.js + Solana Wallet Adapter         |
| Infra            | AWS (EC2/Lambda), Solana devnet RPC     |

## Key Documents

- PRD: `docs/PRD.md`
- Architecture: `docs/architecture/mvp-system-overview.md`

## Agent Behavior

1. Always check MVP scope before implementing new features
2. Never commit secrets (keypairs, `.env`, RPC URLs with embedded keys)
3. All fund movements must go through on-chain programs — no off-chain transfers
4. Oracle data must pass staleness and confidence checks before triggering payout
5. ZK proofs must match the PRD payload schema
6. Do NOT commit to git unless explicitly asked by the user
