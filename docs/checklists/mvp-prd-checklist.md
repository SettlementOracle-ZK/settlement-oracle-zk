# MVP vs PRD checklist (Phase 5.3)

| PRD requirement | MVP status |
|-----------------|------------|
| Single Pyth oracle feed | Done — SOL/USD via Hermes + on-chain legacy account |
| Escrow + automated payout | Done — Anchor `programs/escrow` |
| Staleness / confidence gates | Done — oracle-connector + on-chain `evaluate_trigger` |
| Basic ZK trigger attestation | Done — `zk-prover` commitment hash + API `/verify` |
| Web dashboard (policies, explorer, monitor) | Done — `web/` |
| API PRD payload | Done — `/verify/:hash`, `/settlements/:id` |
| Devnet deployment | Documented — [`devnet-smoke.md`](../runbooks/devnet-smoke.md) |

## Explicitly out of MVP

- Multi-oracle aggregation / dispute windows
- LLMs in ZK circuits
- Mainnet deployment
- Full on-chain SNARK verifier
- AWS production infra
- Third-party insurer core integrations

## Open post-MVP

- Replace commitment hasher with Light Protocol / Succinct SNARK
- Store proof hash on-chain in settlement account
- Continuous monitoring / alerting product
