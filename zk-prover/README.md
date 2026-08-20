# ZK Prover

Off-chain prover that attests parametric settlement rule evaluation against oracle data.

## Status

**MVP implemented** — deterministic SHA-256 commitment circuit (see [`CIRCUIT.md`](CIRCUIT.md)).
Same witness always yields the same `proof_hash`. Full SNARK (Light / Succinct) can replace
the hasher without changing the PRD payload shape.

## Quick start

```bash
cd zk-prover
npm install
npm test
```

## Usage

```typescript
import { generateProof, verifyProof } from './src/index.js';

const proof = generateProof({
  feedId: '0xef0d...',
  oraclePrice: 87.2,
  oracleConf: 0.5,
  publishTime: 1700000000,
  threshold: 100,
  operator: 'lt',
  assetClass: 'agriculture_climate',
  verificationBaseUrl: 'http://127.0.0.1:3000',
});

console.log(proof.proof_hash, proof.payload);
console.log(verifyProof(proof.witness, proof.proof_hash).verified); // true
```

## Layout

```
zk-prover/
├── CIRCUIT.md
├── src/
│   ├── circuit.ts      # rule evaluation (mirrors oracle-connector)
│   ├── commitment.ts   # canonical JSON + SHA-256
│   ├── prove.ts        # PRD payload builder
│   └── verify.ts
└── tests/
```

## Review checklist (task 3.9)

- [x] `CIRCUIT.md` documents inputs/outputs
- [x] Deterministic hash for identical witness
- [x] PRD payload fields (`asset_class`, `risk_score`, `zk_proof`, …)
- [x] No LLM inside circuit
- [x] API can verify via `circuit_commitment` when witness matches hash

## Related

- Oracle inputs: [`../oracle-connector/`](../oracle-connector/)
- API verify: [`../api/`](../api/) — `GET /verify/:proofHash`
- E2E script: [`../scripts/run-settlement-flow.ts`](../scripts/run-settlement-flow.ts)
