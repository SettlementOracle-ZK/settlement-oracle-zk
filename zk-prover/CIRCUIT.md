# Settlement Trigger Circuit (MVP)

MVP uses a **deterministic commitment** over oracle inputs and rule parameters.
Same witness always yields the same `proof_hash`. This attests that trigger
evaluation was performed consistently; a full SNARK (Light Protocol / Succinct)
can replace the hasher without changing the PRD payload shape.

## Stack (MVP)

| Piece | Choice |
|-------|--------|
| Prover host | TypeScript (`zk-prover/`) |
| Commitment | SHA-256 over canonical JSON witness |
| On-chain (MVP) | Store hash off-chain / in API index only |
| Future | Replace `hashWitness()` with SNARK prove/verify |

## Witness (circuit inputs)

All fields are included in the commitment for MVP transparency.

| Field | Type | Source |
|-------|------|--------|
| `feed_id` | string | Pyth feed id (hex) |
| `oracle_price` | number | Normalized price (same units as oracle-connector) |
| `oracle_conf` | number | Normalized confidence |
| `publish_time` | number | Unix seconds |
| `threshold` | number | Policy trigger threshold |
| `operator` | `"lt"` \| `"lte"` \| `"gt"` \| `"gte"` | Rule operator |
| `triggered` | boolean | Rule outcome after staleness/confidence gates |
| `risk_score` | number | 0–100 (same formula as oracle-connector) |
| `asset_class` | string | PRD domain class |

## Public outputs

| Output | Type | Notes |
|--------|------|--------|
| `triggered` | boolean | Exposed in API `public_inputs` |
| `risk_score` | number | PRD `risk_score` |
| `proof_hash` | `0x` + 64 hex | SHA-256 commitment |

## Evaluation logic

Mirrors `oracle-connector/src/evaluateTrigger.ts`:

1. Reject `price === 0`
2. Reject stale (`publish_time` older than 60s vs evaluation time)
3. Reject low confidence (`conf / |price| > 0.05`)
4. Compare price vs threshold using `operator`
5. Compute `risk_score = min(100, round(|price/threshold| * 100))`

## PRD payload mapping

| PRD field | Source |
|-----------|--------|
| `asset_class` | witness `asset_class` |
| `risk_score` | witness `risk_score` |
| `scale` | constant `"0-100"` |
| `model_confidence` | `100 - round(conf/|price|*100)` capped, formatted as `"NN%"` |
| `timestamp` | ISO-8601 UTC at proof generation |
| `zk_proof.hash` | `proof_hash` |
| `zk_proof.verification_url` | `{API_PUBLIC_BASE_URL}/verify/{hash}` |

## Verification

Off-chain verify recomputes SHA-256 over the stored witness JSON (keys sorted,
stable number formatting) and compares to `proof_hash`. API sets
`verification_method: "circuit_commitment"` and `verified: true` when the
recomputed hash matches.

## Non-goals (MVP)

- LLMs or ML inside the circuit
- Trusted setup / ceremony
- On-chain SNARK verifier program
