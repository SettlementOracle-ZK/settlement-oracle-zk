---
name: sdd-week-orchestrator
description: >-
  Orchestrates SettlementOracle ZK MVP development using Spec-Driven Development (SDD)
  aligned to the current week/phase in docs/plans/mvp-dev-plan.md. Use when starting a
  dev session, asking "what should I work on this week", running SDD for phase 0-5,
  or orchestrating tasks for Rodrigo or Klisman.
---

# SDD Week Orchestrator

Guide MVP development week-by-week using **Spec-Driven Development**: spec → design slice → deliver → verify against the phase checkpoint.

## Source of Truth (read every session)

1. `docs/plans/mvp-dev-plan.md` — phase tasks, ownership (P/S/J), checkpoints
2. `docs/PRD.md` — product requirements and payload schema
3. `AGENTS.md` — MVP IN/OUT boundaries and layer map
4. Layer rule for the task: `.cursor/rules/<layer>.mdc`
5. Layer skill if applicable: `.cursor/skills/<skill>/SKILL.md`

For phase details and checkpoints, see [phases.md](phases.md).

---

## Step 1 — Establish context

If not provided, ask **one question at a time**:

1. **Phase/week** (0–5)
2. **Developer** (`Rodrigo` | `Klisman`)
3. **Intent** — pick one:
   - `next` — next unchecked Primary task for this developer in this phase
   - `<task-id>` — e.g. `1.1`, `2.6`
   - `checkpoint` — verify phase completion only
   - `review` — Secondary tasks for partner's open PR

---

## Step 2 — SDD: Spec

For the selected task(s):

1. Quote the task row from `mvp-dev-plan.md` (ID, P/S/J, layer, description).
2. Map to PRD requirement (section + user story if relevant).
3. Confirm **MVP IN scope** — if ambiguous, invoke `mvp-feature-scoping` logic before proceeding.
4. List **acceptance criteria** derived from the phase checkpoint.
5. Note **dependencies** — block if upstream tasks from prior phases are incomplete.

### On-chain ownership (50/50)

| Area | Rodrigo | Klisman |
|------|---------|---------|
| Escrow + deposit + `evaluate_trigger` | **P** | S |
| Policy + errors + payout + pause | S | **P** |
| Deploy build tx | **P** | verify + smoke txs |

Do not assign or implement on-chain work outside this split unless the user explicitly overrides.

---

## Step 3 — SDD: Design slice

Before writing code, produce a short design (scale to task complexity):

```markdown
## Design: <task-id> — <title>

**Spec ref:** mvp-dev-plan §Fase N, PRD §X
**Owner:** <developer> (P|S|J)
**Files:** create/modify list with paths
**Interface contract:** inputs/outputs crossing layer boundaries
**Test plan:** commands + scenarios
**Out of scope:** what this task must NOT include
```

For **Primary (P)** tasks with cross-layer impact, wait for user approval before implementing.

For **Secondary (S)** review tasks, output a review checklist instead of a design.

---

## Step 4 — SDD: Deliver

Implementation rules:

| Rule | Detail |
|------|--------|
| Branch | `feat/<layer>-<short-desc>` |
| PR size | ≤ ~300 lines; Primary opens PR |
| On-chain PRs | Reviewer runs `anchor test` before approve |
| Secrets | Never commit keypairs or `.env` |
| Infra | Local first — no AWS for MVP |
| Git | Do NOT commit unless user explicitly asks |

Invoke layer skills when relevant:

| Layer | Skill |
|-------|-------|
| `programs/` | `anchor-program-dev` |
| `oracle-connector/` | `pyth-oracle-integration` |
| `zk-prover/` | `zk-proof-generation` |
| devnet deploy | `solana-devnet-deploy` |
| domain questions | `settlement-oracle-domain` |

Track progress in session output:

```
Task Progress:
- [ ] Spec aligned
- [ ] Design approved
- [ ] Implementation
- [ ] Tests passing
- [ ] Checkpoint criteria met
```

---

## Step 5 — SDD: Verify

Run verification appropriate to the layer:

| Layer | Verify with |
|-------|-------------|
| `programs/` | `anchor test` |
| `oracle-connector/` | unit tests (staleness, confidence edge cases) |
| `api/` | `GET /health`, endpoint smoke test |
| `zk-prover/` | deterministic proof hash for same inputs |
| `web/` | page renders + wallet connects |
| Phase 4 | devnet smoke: init → deposit → trigger → payout |

Report checkpoint status:

```markdown
## Phase <N> Checkpoint

- [ ] Criterion 1
- [ ] Criterion 2
**Status:** PASS | PARTIAL | BLOCKED
**Blockers:** ...
**Partner sync needed:** yes/no
```

If **BLOCKED**, stop and list what must be done first (prior phase task or partner PR).

---

## Session output template

Use this format when orchestrating a session:

```markdown
# SDD Session — Phase <N>, Week <N>, <Developer>

## Active task
<id> | <P/S/J> | <layer> | <description>

## Spec alignment
- PRD: ...
- MVP scope: IN
- Checkpoint contribution: ...

## Design slice
(concise design or review checklist)

## Next actions
1. ...
2. ...

## Partner handoff
What <other developer> needs from this task: ...
```

---

## Phase quick reference

| Phase | Week | Focus | Checkpoint |
|-------|------|-------|------------|
| 0 | 0 | Scaffold + env setup (J) | Repo + Anchor workspace on both machines |
| 1 | 1 | Escrow (R) + policy (K) + oracle (K) | Accounts exist; Pyth feed validated |
| 2 | 2 | `evaluate_trigger` (R) + payout (K) + API (K) | On-chain loop + API reads escrow |
| 3 | 3 | ZK (R) + dashboard (K) | Proof explorer + `/verify` endpoint |
| 4 | 4 | Deploy (R) + E2E integration (K) | Devnet demo, both ran on-chain txs |
| 5 | 5 | Demo + hardening (J) | E2E recording + MVP vs PRD checklist |

Full task tables: `docs/plans/mvp-dev-plan.md`

---

## Examples

**User:** "I'm Rodrigo, week 1, what's next?"
→ Phase 1, filter Rodrigo **P** tasks → start with `1.1` if unchecked → SDD spec/design for `EscrowAccount` + `initialize_escrow`.

**User:** "Klisman, phase 2, task 2.6"
→ Spec for `execute_payout` + `pause`/`unpause` → design → implement → `anchor test` → verify contributes to Phase 2 checkpoint.

**User:** "Phase 1 checkpoint"
→ Read checkpoint criteria → audit repo state → report PASS/PARTIAL/BLOCKED for both developers' deliverables.

**User:** "Rodrigo, week 2, review mode"
→ List Klisman **P** tasks needing Rodrigo **S** review (`2.6`, `2.7`) → output review checklist per on-chain PR rules.
