---
name: pyth-oracle-integration
description: >-
  Integrate Pyth Network price feeds into SettlementOracle ZK oracle connector.
  Use when consuming Pyth feeds, implementing staleness checks, mapping feed
  values to trigger conditions, or testing oracle-connector module.
---

# Pyth Oracle Integration

## Workflow

```
Task Progress:
- [ ] Step 1: Select MVP feed
- [ ] Step 2: Implement feed reader
- [ ] Step 3: Add staleness + confidence checks
- [ ] Step 4: Map to trigger condition
- [ ] Step 5: Test with devnet feed
```

**Step 1: Select feed**
- Browse Pyth feed IDs at https://pyth.network/developers/price-feed-ids
- Document chosen feed ID in `oracle-connector/README.md`
- MVP: pick ONE feed (climate OR price)

**Step 2: Feed reader**
```typescript
import { PriceServiceConnection } from '@pythnetwork/price-service-client';

const connection = new PriceServiceConnection('https://hermes.pyth.network');
const priceFeed = await connection.getLatestPriceFeeds([FEED_ID]);
```

**Step 3: Validation**
- Reject if `publishTime` older than 60 seconds
- Reject if `confidence / price > 0.05`

**Step 4: Trigger mapping**
- Define `TriggerRule { threshold, operator }` per policy
- Return `{ triggered, risk_score, timestamp }` for ZK prover input

**Step 5: Test**
- Use Pyth devnet feeds — do not mock oracle data in integration tests
