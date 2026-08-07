# Oracle Connector

Service module that consumes high-fidelity market oracle feeds (MVP: **Pyth Network** on Solana) and exposes validated data for trigger evaluation.

## Status

**Scaffold only — not implemented**

## Purpose

- Read Pyth price / climate (or other chosen MVP feed class) accounts on Solana
- Enforce **staleness** and **confidence** checks before data is treated as trigger-ready
- Produce structured inputs for the escrow program and ZK prover

## Responsibilities

| Area | Description |
|------|-------------|
| Feed client | Fetch and parse Pyth on-chain accounts |
| Validation | Reject stale, low-confidence, or malformed readings |
| Trigger inputs | Map feed values to policy thresholds / strike conditions |
| Integration surface | Stable interface for API / workers that evaluate settlement |

## Target stack

- **Language:** TypeScript (or Rust client if co-located with programs later)
- **Oracle (MVP):** Pyth Network on Solana
- **Network (MVP):** Devnet / testnet RPC

## Future layout (when implemented)

```
oracle-connector/
├── package.json          # or Cargo.toml if Rust
├── src/
│   ├── client/
│   ├── validation/
│   ├── feeds/
│   └── index.ts
└── tests/
```

## Non-goals (MVP)

- Multi-oracle aggregation (Chainlink + Pyth + API3)
- Dispute / challenge windows before settlement
- Continuous monitoring and alerting pipeline
- Writing oracle data on-chain (read path only for MVP)

## Related

- Product requirements: [`../docs/PRD.md`](../docs/PRD.md)
- Agent conventions: [`../AGENTS.md`](../AGENTS.md)
- Cursor rule: `oracle-integration` / skill: `pyth-oracle-integration`
- Downstream: [`../programs/escrow/`](../programs/escrow/), [`../zk-prover/`](../zk-prover/), [`../api/`](../api/)
