# Oracle Connector

Pyth Network feed consumer for SettlementOracle ZK MVP trigger evaluation.

## MVP feed (chosen)

| Field | Value |
|-------|-------|
| Asset | SOL/USD |
| Feed ID | `0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d` |
| Hermes URL | `https://hermes.pyth.network` |
| Network | Devnet / mainnet (read-only via Hermes) |

## Validation defaults

- **Staleness:** reject if `publish_time` older than **60s**
- **Confidence:** reject if `conf / |price| > 0.05`

## Layout

```
oracle-connector/
├── src/
│   ├── client/pyth.ts       # Pyth Hermes client
│   ├── constants.ts
│   ├── evaluateTrigger.ts   # Trigger evaluation (fail closed)
│   ├── types.ts
│   ├── validation.ts
│   └── index.ts
└── tests/
    ├── staleness.test.ts
    ├── confidence.test.ts
    └── evaluateTrigger.test.ts
```

## Run locally

```bash
cd oracle-connector
npm install
npm test
```

## Usage

```typescript
import { evaluateTrigger, PythHermesClient } from './src/index.js';

const client = new PythHermesClient();
const feed = await client.getLatestPriceFeed();
const result = evaluateTrigger(feed, { threshold: 100, operator: 'lt' });
```

## Related

- PRD: [`../docs/PRD.md`](../docs/PRD.md)
- Dev plan Phase 1: [`../docs/plans/mvp-dev-plan.md`](../docs/plans/mvp-dev-plan.md)
