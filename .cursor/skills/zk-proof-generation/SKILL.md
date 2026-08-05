---
name: zk-proof-generation
description: >-
  Generate and verify ZK proofs for SettlementOracle ZK trigger attestation.
  Use when building ZK circuits, generating proofs from oracle+rule inputs,
  exposing verification endpoints, or referencing proof hashes on-chain.
---

# ZK Proof Generation

## MVP Scope
Prove: given oracle data (price, timestamp, confidence) and rule params
(threshold), the trigger evaluation result is correct.

Do NOT include LLMs or complex ML models in the circuit.

## Workflow

```
Task Progress:
- [ ] Step 1: Define circuit inputs/outputs
- [ ] Step 2: Implement circuit
- [ ] Step 3: Generate proof from trigger evaluation
- [ ] Step 4: Expose verification endpoint
- [ ] Step 5: Store hash on-chain
```

**Step 1: Circuit I/O**
- Private inputs: oracle price, confidence, publish_time, threshold
- Public outputs: triggered (bool), risk_score (u64)

**Step 2: Implement**
- Use Light Protocol or Succinct (pick one, document in `zk-prover/CIRCUIT.md`)
- Circuit must be deterministic

**Step 3: Generate**
```typescript
const proof = await prover.generate({
  oraclePrice, confidence, publishTime, threshold
});
// Returns: { hash, verification_url }
```

**Step 4: Verification endpoint**
- `GET /verify/:hash` — returns full proof + public inputs
- Response must match PRD payload schema

**Step 5: On-chain reference**
- Store `proof.hash` in settlement account on-chain
- Full verification is off-chain for MVP
