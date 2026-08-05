---
name: settlement-oracle-domain
description: >-
  Parametric insurance domain knowledge for SettlementOracle ZK. Covers trigger
  evaluation, oracle reliability, escrow/payout flow, and ZK attestation. Use when
  discussing business rules, claim triggers, oracle risk, payout logic, or parametric
  insurance concepts.
---

# Settlement Oracle Domain

## Core Flow

```
Policy Created → Premium Escrowed → Oracle Monitors Event
                                          ↓
                              Trigger Condition Met?
                                    ↓ YES
                              ZK Proof Generated
                                    ↓
                              Payout Released On-chain
```

## Key Concepts

- **Parametric insurance:** Payout based on index/event data, not loss assessment
- **Trigger:** Boolean condition on oracle data (e.g., rainfall < 50mm)
- **Escrow:** On-chain vault holding premium until trigger or expiry
- **ZK attestation:** Cryptographic proof that trigger rule was applied correctly

## Oracle Risk Mitigations (from PRD)
- Staleness check before every trigger evaluation
- Confidence interval validation
- Fail closed: reject payout if oracle data is unreliable

## PRD Payload Fields
| Field | Meaning |
|-------|---------|
| `asset_class` | Insurance vertical (e.g., `agriculture_climate`) |
| `risk_score` | 0-100 score from trigger evaluation |
| `model_confidence` | Confidence in the evaluation |
| `zk_proof.hash` | On-chain reference to full proof |
| `zk_proof.verification_url` | Off-chain full verification endpoint |

## MVP Use Case (pick one)
- **Agriculture/climate:** Pyth weather feed → drought trigger
- **Price protection:** Pyth price feed → strike price trigger
