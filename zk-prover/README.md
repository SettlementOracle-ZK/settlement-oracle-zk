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

## Review checklist for circuit PR (task 3.9)

Blocked until Rodrigo opens the 3.1–3.3 PR. When it lands, review against:

- [ ] `CIRCUIT.md` documents public vs private inputs (oracle price, confidence, publish_time, threshold → `triggered`, `risk_score`)
- [ ] Same inputs always produce the same proof hash (deterministic)
- [ ] Payload matches PRD: `asset_class`, `risk_score`, `scale`, `model_confidence`, `timestamp`, `zk_proof.hash`, `zk_proof.verification_url`
- [ ] No LLM / large model inside the circuit
- [ ] Proof hash can be stored in `proofs.proof_hash` and served by `GET /verify/:proofHash`
- [ ] Unit test: identical witness → identical hash

Klisman 3.10 ships `/verify` as a stored-attestation lookup so the dashboard is unblocked. Swap `verification_method` to circuit-native verify without changing the PRD JSON shape.

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
