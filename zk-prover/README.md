# ZK Prover

Off-chain prover that generates a zero-knowledge proof attesting that a parametric settlement rule was executed correctly against oracle data.

## Status

**Scaffold only — not implemented**

## Purpose

- Accept private/public inputs describing the rule evaluation (oracle reading, threshold, outcome)
- Produce a ZK-SNARK (or equivalent) suitable for audit and API exposure
- Align proof metadata with the PRD payload schema (`zk_proof.hash`, verification URL)

## Responsibilities

| Area | Description |
|------|-------------|
| Witness building | Construct prover inputs from oracle + policy parameters |
| Proof generation | Run the selected ZK stack for the settlement circuit |
| Verification helpers | Local or remote verify path for auditors and the API |
| Schema alignment | Emit proof artifacts consumable by `api` and `web` |

## Target stack

- **ZK (MVP candidates):** Light Protocol or Succinct (Solana-compatible path)
- **Host:** Off-chain service (not LLMs inside the circuit)
- **Network (MVP):** Devnet-aligned integration tests only

## Future layout (when implemented)

```
zk-prover/
├── package.json or Cargo.toml
├── circuits/ or guest/       # depends on ZK stack
├── src/
│   ├── prove/
│   ├── verify/
│   └── types/
└── tests/
```

## Non-goals (MVP)

- Large models (LLMs) inside the ZK circuit
- Multi-proof batching infrastructure
- Mainnet-grade ceremony / production key management
- Continuous proof pipelines without an explicit settlement event

## Related

- Product requirements: [`../docs/PRD.md`](../docs/PRD.md) (payload schema under functional requirements)
- Architecture: [`../docs/architecture/mvp-system-overview.md`](../docs/architecture/mvp-system-overview.md)
- Shared contracts: [`../shared/README.md`](../shared/README.md)
- Agent conventions: [`../AGENTS.md`](../AGENTS.md)
- Cursor rule: `zk-proofs` / skill: `zk-proof-generation`
- Inputs from: [`../oracle-connector/`](../oracle-connector/), [`../programs/escrow/`](../programs/escrow/)
- Consumers: [`../api/`](../api/), [`../web/`](../web/)
