---
name: mvp-feature-scoping
description: >-
  Evaluate whether a requested feature is within SettlementOracle ZK MVP scope.
  Use when adding new features, evaluating scope creep, or when the user asks
  "should we add..." for capabilities not explicitly in the MVP plan.
---

# MVP Feature Scoping

## Workflow

1. Read the feature request
2. Check against MVP IN/OUT lists in `AGENTS.md`
3. Respond with one of:

**IN scope → proceed:**
> "This feature is within MVP scope. Proceeding."

**OUT of scope → block:**
> "This feature is OUT of MVP scope (<reason>). I recommend deferring to post-MVP. Proceed anyway?"

**Ambiguous → ask:**
> "This isn't explicitly listed in the MVP scope. Should I implement it now or defer?"

## MVP IN (quick reference)
- Single Pyth oracle feed
- Escrow + payout Anchor programs (devnet)
- Basic ZK proof for trigger
- Web dashboard (monitoring)
- API Gateway with PRD payload schema

## MVP OUT (quick reference)
- Multi-oracle aggregation
- LLMs in ZK circuits
- Mainnet deployment
- Continuous monitoring/alerting
- Third-party insurer system integration
- Challenge/dispute periods
