# Phase Reference — SettlementOracle ZK MVP

Synced with `docs/plans/mvp-dev-plan.md`. Read the plan file for full task tables.

---

## Phase 0 — Alignment (Week 0) · Joint

**Goal:** Monorepo scaffold, Anchor workspace, local env on both machines.

**Tasks (all J):**
- Choose Pyth feed (price recommended for MVP)
- Scaffold `programs/`, `oracle-connector/`, `zk-prover/`, `api/`, `web/`
- Anchor workspace (`Anchor.toml`, `programs/escrow/`) — pair session
- `docker-compose.yml` + `.env.example`
- Solana + Anchor setup, devnet wallets + airdrop
- Sync ritual: 2x/week, 30 min

**Checkpoint:** Repo scaffold + Anchor workspace working on both laptops.

**Gate before Phase 1:** Feed ID chosen and documented.

---

## Phase 1 — Foundation (Week 1)

**Goal:** On-chain accounts + oracle reader.

| Dev | Primary | Secondary |
|-----|---------|-----------|
| Rodrigo | 1.1–1.2 escrow + deposit | 1.3–1.4 review + oracle tests |
| Klisman | 1.5–1.8 policy + oracle | 1.9–1.10 review + integration stub |

**Checkpoint:** Escrow + policy accounts exist; oracle reads Pyth feed with staleness/confidence validation.

**Layer skills:** `anchor-program-dev`, `pyth-oracle-integration`

---

## Phase 2 — Core Loop (Week 2)

**Goal:** Trigger + payout on-chain; API exposes escrow state.

| Dev | Primary | Secondary |
|-----|---------|-----------|
| Rodrigo | 2.1–2.2 `evaluate_trigger` + tests | 2.3–2.5 review payout + API scaffold |
| Klisman | 2.6–2.9 payout + API schema/RPC | 2.10–2.11 review trigger + submit script |

**Checkpoint:** Trigger + payout tested on-chain; API exposes escrow status.

**Layer skills:** `anchor-program-dev`, `pyth-oracle-integration`

**Gate before Phase 3:** `GET /policies/:id` returns real on-chain data.

---

## Phase 3 — ZK + Frontend (Week 3)

**Goal:** Proof generation + dashboard views.

| Dev | Primary | Secondary |
|-----|---------|-----------|
| Rodrigo | 3.1–3.3 ZK circuit + PRD payload | 3.4–3.5 Next.js scaffold + policies view |
| Klisman | 3.6–3.8 explorer + monitor + wallet | 3.9–3.10 review circuit + `/verify` |

**Checkpoint:** Dashboard shows policies + proof explorer; API verifies ZK hash.

**Layer skills:** `zk-proof-generation`

**Gate before Phase 4:** `GET /verify/:proofHash` returns PRD payload shape.

---

## Phase 4 — Deploy + E2E (Week 4)

**Goal:** Devnet deploy + full local E2E demo.

| Dev | Primary | Secondary |
|-----|---------|-----------|
| Rodrigo | 4.1–4.3 deploy + smoke pt.1 + arch doc | 4.4–4.5 dashboard devnet config |
| Klisman | 4.6–4.10 smoke pt.2 + settlements + E2E + README | 4.11 dashboard E2E test |

**Checkpoint:** E2E demo on devnet; both developers executed on-chain txs in smoke tests.

**Layer skills:** `solana-devnet-deploy`

**Gate before Phase 5:** Program ID documented in `docs/architecture/mvp-system-overview.md`.

---

## Phase 5 — Demo + Hardening (Week 5) · Joint

**Goal:** Recorded demo, security review, MVP checklist.

| Task | Owner |
|------|-------|
| 5.1 E2E recording | J |
| 5.2 On-chain security review | J |
| 5.3 MVP vs PRD checklist | J |
| 5.4 Stakeholder demo prep | J |
| 5.5 On-chain bug fixes | J |
| 5.6 Off-chain + UI bug fixes | Klisman P / Rodrigo S |

**Checkpoint:** Demo ready; MVP scope documented as delivered vs deferred.

---

## Dependency chain

```
Phase 0 → Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5
              ↑ parallel: escrow (R) ↔ policy (K) ↔ oracle (K)
```

**Mandatory sync sessions** before starting phases 2, 3, and 4.

---

## Infrastructure reminder

- **Local first** for all off-chain components
- **Only mandatory deploy:** Anchor programs to Solana devnet
- **AWS:** post-MVP
