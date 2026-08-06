# Shared Contracts

Cross-cutting **schemas and type contracts** used by more than one package: API responses, ZK proof metadata, policy identifiers, and related domain DTOs.

## Status

**Scaffold only — not implemented**

No generated types, schemas, or crates yet. This package is the intended home for shared contracts so consumers stay aligned.

## Purpose

- Single definition of the PRD settlement / risk payload
- Stable identifiers (policy, settlement, proof hash formats)
- Shared vocabulary for `api`, `zk-prover`, `web`, and indexer code
- Avoid divergent JSON field names and silent Breaking changes across packages

## Consumers

| Package | How it uses shared contracts |
|---------|------------------------------|
| [`api/`](../api/) | Serialize/deserialize HTTP responses and request params |
| [`zk-prover/`](../zk-prover/) | Proof hash and public output shapes |
| [`web/`](../web/) | Typed clients and explorer UI models |
| [`oracle-connector/`](../oracle-connector/) | Optional trigger-input DTOs when passed across process boundaries |
| [`programs/escrow/`](../programs/escrow/) | Does **not** depend on this package at runtime (on-chain state is Anchor/IDL); may document PDA field mapping here later |

## Canonical settlement payload (PRD)

```json
{
  "asset_class": "agriculture_climate",
  "risk_score": 85.4,
  "scale": "0-100",
  "model_confidence": "92%",
  "timestamp": "2026-05-19T14:42:00Z",
  "zk_proof": {
    "hash": "0xABC123...",
    "verification_url": "https://api.riskoracle.com/verify/0xABC..."
  }
}
```

| Field | Notes |
|-------|--------|
| `asset_class` | Domain class string (e.g. `agriculture_climate`) |
| `risk_score` | Numeric score on `scale` |
| `scale` | Score bounds descriptor (MVP: `"0-100"`) |
| `model_confidence` | Confidence string (format may be tightened later) |
| `timestamp` | ISO-8601 UTC |
| `zk_proof.hash` | Proof digest for audit / on-chain reference (MVP) |
| `zk_proof.verification_url` | Off-chain verification endpoint |

## Related API surface (for schema alignment)

Mirror endpoint params/bodies once implemented (see `.cursor/rules/api-gateway.mdc`):

- `GET /policies/:id`
- `GET /settlements/:id`
- `GET /verify/:proofHash`

## Target stack

Prefer **schema-first** definitions consumable by both Rust and TypeScript:

| Artifact (planned) | Use |
|--------------------|-----|
| JSON Schema and/or OpenAPI | Human + machine source of truth |
| Rust types (`serde`) | Generated or mirrored into a small crate used by `api` / provers |
| TypeScript types | Generated for `web` (and other TS clients) |

MVP does not require choosing codegen tools yet; document the choice in an ADR when implementing.

## Future layout (when implemented)

```
shared/
├── README.md                 # this file
├── openapi/                  # optional: OpenAPI fragments or full gateway spec
│   └── settlement-v1.yaml
├── schemas/                  # JSON Schema for DTOs
│   └── settlement-payload.json
└── (later) rust/ or packages/  # language bindings if not monorepo-generated elsewhere
```

## Non-goals (MVP)

- Business logic, RPC clients, or database models
- On-chain program bytecode or Anchor instruction builders
- Secrets, env templates, or network config
- Multiple competing payload versions without deprecation notes

## Versioning

Until `1.0`, treat field renames as breaking across all consumers. Prefer additive fields. Document changes in this README or an ADR.

## Related

- Architecture: [`../docs/architecture/mvp-system-overview.md`](../docs/architecture/mvp-system-overview.md)
- Product requirements: [`../docs/PRD.md`](../docs/PRD.md)
- Agent conventions: [`../AGENTS.md`](../AGENTS.md)
- API package: [`../api/README.md`](../api/README.md)
- ZK package: [`../zk-prover/README.md`](../zk-prover/README.md)
