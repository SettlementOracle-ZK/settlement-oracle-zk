# Plano de Desenvolvimento — SettlementOracle ZK (MVP)

**Time:** Rodrigo + Klisman  
**Premissa:** cada um **lidera** camadas diferentes, mas **mexe em todas** via tarefas secundárias (S)  
**On-chain:** split **50/50** — Rodrigo owns `escrow` + `evaluate_trigger`; Klisman owns `policy`/errors + `execute_payout` + `pause`  
**Infra:** local first — único deploy obrigatório = programas Anchor na devnet  
**Duração estimada:** 4–5 semanas (part-time) ou 2–3 semanas (full-time)

---

## Legenda

| Sigla | Significado |
|-------|-------------|
| **P** | Primary — dono da entrega, abre o PR |
| **S** | Secondary — revisa, integra, faz tarefa menor na mesma camada |
| **J** | Joint — fazem juntos na mesma sessão |

---

## Split on-chain (referência rápida)

| Área on-chain | Rodrigo | Klisman |
|---------------|---------|---------|
| Scaffold Anchor workspace | **J** Fase 0 | **J** Fase 0 |
| `EscrowAccount` + `initialize_escrow` + `deposit_premium` | **P** | S (review) |
| `PolicyAccount` + `error.rs` compartilhado | S (review) | **P** |
| `evaluate_trigger` | **P** | S (review + test script) |
| `execute_payout` + `pause`/`unpause` | S (review) | **P** |
| Testes on-chain | **P** trigger path | **P** payout + circuit breaker |
| Deploy devnet | **P** build + deploy tx | **P** verify + smoke on-chain |
| Bug fixes on-chain (Fase 5) | **J** | **J** |

---

## Fase 0 — Alinhamento (Semana 0) · **J**

Antes de codar, vocês dois fecham:

- [ ] **J** Escolher use case MVP: feed de **preço** ou **clima** (Pyth devnet)
- [ ] **J** Scaffold monorepo (`programs/`, `oracle-connector/`, `zk-prover/`, `api/`, `web/`)
- [ ] **J** Scaffold Anchor workspace (`Anchor.toml`, `programs/escrow/`) — **pair session**
- [ ] **J** `docker-compose.yml` com PostgreSQL local
- [ ] **J** `.env.example` compartilhado (RPC devnet, DB, feed ID)
- [ ] **J** Setup Solana + Anchor nas duas máquinas (`anchor --version`, `solana --version`)
- [ ] **J** Carteiras devnet + airdrop SOL (cada um com a sua)
- [ ] **J** Definir ritual de sync: 2x/semana, 30 min + canal para blockers

**Entregável:** repo scaffold + Anchor workspace + ambiente funcionando nos dois laptops.

---

## Fase 1 — Fundação (Semana 1)

### Rodrigo · Primary escrow (on-chain) | Secondary oracle

| # | Tarefa | Camada |
|---|--------|--------|
| 1.1 | **P** `EscrowAccount` struct + `initialize_escrow` instruction | `programs/` |
| 1.2 | **P** Instruction `deposit_premium` + testes unitários | `programs/` |
| 1.3 | **S** Revisar PR do Klisman (`PolicyAccount`, `error.rs`) | `programs/` |
| 1.4 | **S** Escrever 2 testes de staleness/confidence no oracle (pair programming ok) | `oracle-connector/` |

### Klisman · Primary oracle + policy (on-chain) | Secondary escrow

| # | Tarefa | Camada | Status |
|---|--------|--------|--------|
| 1.5 | **P** `PolicyAccount` struct + `error.rs` compartilhado + testes | `programs/` | Done (#3) |
| 1.6 | **P** Scaffold `oracle-connector/` (TypeScript) | `oracle-connector/` | Done (#4) |
| 1.7 | **P** Integrar Pyth Hermes — ler feed escolhido na Fase 0 | `oracle-connector/` | Done (#4) |
| 1.8 | **P** `evaluateTrigger()` com checks de staleness + confidence | `oracle-connector/` | Done (#4) |
| 1.9 | **S** Revisar PR do Rodrigo (`initialize_escrow`, `deposit_premium`) | `programs/` | Done (review on `main`) |
| 1.10 | **S** Integration test stub: `initialize_escrow` → assert account state | `programs/` | Done (`test_phase1.rs`) |

**Checkpoint Fase 1:** escrow + policy accounts existem; oracle lê feed Pyth com validação. Klisman 1.5–1.10 complete. Next: Fase 2.6 `execute_payout` + `pause`/`unpause`.

---

## Fase 2 — Core loop (Semana 2)

### Rodrigo · Primary `evaluate_trigger` (on-chain) | Secondary API

| # | Tarefa | Camada | Status |
|---|--------|--------|--------|
| 2.1 | **P** Instruction `evaluate_trigger` (CPI Pyth on-chain ou flag off-chain) | `programs/` | Done |
| 2.2 | **P** Testes on-chain: trigger happy path + oracle stale → sem payout | `programs/` | Done |
| 2.3 | **S** Revisar PR payout do Klisman (`execute_payout`, `pause`) | `programs/` | Done (merged) |
| 2.4 | **S** Scaffold `api/` (rust + PostgreSQL) | `api/` | Done (Klisman; unblocked 2.8/2.9) |
| 2.5 | **S** Endpoint `GET /health` + conexão DB | `api/` | Done (Klisman; unblocked 2.8/2.9) |

### Klisman · Primary `execute_payout` (on-chain) + API | Secondary trigger

| # | Tarefa | Camada | Status |
|---|--------|--------|--------|
| 2.6 | **P** Instruction `execute_payout` + circuit breaker `pause`/`unpause` | `programs/` | Done |
| 2.7 | **P** Testes on-chain: payout happy path + pause bloqueia payout | `programs/` | Done |
| 2.8 | **P** Schema DB: `policies`, `settlements` | `api/` | Done |
| 2.9 | **P** `GET /policies/:id` + integração RPC Solana (ler escrow) | `api/` | Done |
| 2.10 | **S** Revisar PR `evaluate_trigger` do Rodrigo | `programs/` | Ready (checklist in `programs/escrow/README.md`) |
| 2.11 | **S** Script que submete tx `evaluate_trigger` via `@solana/web3.js` | `scripts/` | Done (exits until 2.1 lands in IDL) |

**Checkpoint Fase 2:** trigger + payout on-chain testados; API expõe status do escrow. Rodrigo 2.1/2.2 + Klisman 2.6–2.9 + 2.11 complete.

---

## Fase 3 — ZK + Frontend (Semana 3)

### Rodrigo · Primary ZK | Secondary frontend

| # | Tarefa | Camada | Status |
|---|--------|--------|--------|
| 3.1 | **P** Scaffold `zk-prover/` + definir circuit I/O em `CIRCUIT.md` | `zk-prover/` | Done |
| 3.2 | **P** Circuit básico: oracle data + threshold → `triggered` + `risk_score` | `zk-prover/` | Done |
| 3.3 | **P** Gerar proof hash + payload no formato do PRD | `zk-prover/` | Done |
| 3.4 | **S** Scaffold Next.js + Solana Wallet Adapter | `web/` | Done (Klisman; unblocked 3.6–3.8) |
| 3.5 | **S** View **Active Policies** (lista mockada → API real) | `web/` | Done (Klisman; unblocked checkpoint) |

### Klisman · Primary frontend | Secondary ZK

| # | Tarefa | Camada | Status |
|---|--------|--------|--------|
| 3.6 | **P** View **Settlement Explorer** (tx hash, proof hash, link verify) | `web/` | Done |
| 3.7 | **P** View **Trigger Monitor** (feed value vs threshold, warning staleness) | `web/` | Done |
| 3.8 | **P** Conectar wallet Phantom/Solflare | `web/` | Done |
| 3.9 | **S** Revisar PR circuit do Rodrigo | `zk-prover/` | Ready (checklist in `zk-prover/README.md`) |
| 3.10 | **S** Endpoint `GET /verify/:proofHash` na API | `api/` + `zk-prover/` | Done (stored attestation; circuit verify waits on 3.1–3.3) |

**Checkpoint Fase 3:** dashboard mostra policies + proof explorer; API verifica hash ZK. Rodrigo 3.1–3.3 + Klisman 3.6–3.8 + 3.10 complete.

---

## Fase 4 — Deploy + E2E (Semana 4)

### Rodrigo · Primary deploy (build + tx) | Secondary frontend

| # | Tarefa | Camada | Status |
|---|--------|--------|--------|
| 4.1 | **P** `anchor build` + deploy devnet (`anchor deploy`) | `programs/` | Documented ([`devnet-smoke.md`](../runbooks/devnet-smoke.md)) |
| 4.2 | **P** Smoke on-chain (parte 1): `initialize_escrow` → `deposit_premium` | `programs/` | Documented |
| 4.3 | **P** Documentar program ID em `docs/architecture/mvp-system-overview.md` | `docs/` | Done |
| 4.4 | **S** Ajustar dashboard para devnet (program ID, RPC) | `web/` | Done |
| 4.5 | **S** Badge de status: `PENDING` / `TRIGGERED` / `PAID` / `FAILED` | `web/` | Done |

### Klisman · Primary smoke on-chain + integração | Secondary deploy verify

| # | Tarefa | Camada | Status |
|---|--------|--------|--------|
| 4.6 | **P** Smoke on-chain (parte 2): `evaluate_trigger` → `execute_payout` | `programs/` + `oracle-connector/` | Documented |
| 4.7 | **P** Verificar deploy (second pair of eyes): `solana program show` | `programs/` | Documented |
| 4.8 | **P** Endpoint `GET /settlements/:id` com payload PRD completo | `api/` | Done |
| 4.9 | **P** Conectar oracle-connector → prover → API → on-chain (fluxo completo) | off-chain | Done (`scripts/run-settlement-flow.ts`) |
| 4.10 | **P** README com instruções "run locally" | root | Done |
| 4.11 | **S** Testar fluxo E2E no dashboard | `web/` | Manual (runbook) |

**Checkpoint Fase 4:** demo E2E documentada; deploy devnet requer execução manual com wallet funded.

---

## Fase 5 — Demo + hardening (Semana 5) · **J**

| # | Tarefa | Quem | Status |
|---|--------|------|--------|
| 5.1 | **J** Teste E2E gravado (screen recording do fluxo completo) | ambos | Manual |
| 5.2 | **J** Revisão de segurança on-chain (PDAs, signers, no unwrap) | ambos | Checklist ([`on-chain-security-review.md`](../checklists/on-chain-security-review.md)) |
| 5.3 | **J** Checklist MVP vs PRD — o que entrou, o que ficou fora | ambos | Done ([`mvp-prd-checklist.md`](../checklists/mvp-prd-checklist.md)) |
| 5.4 | **J** Preparar demo para stakeholders | ambos | Manual |
| 5.5 | **J** Fix bugs on-chain | `programs/` | As needed |
| 5.6 | Klisman **P** / Rodrigo **S** — fix bugs off-chain + UI | `api/` + `web/` | As needed |

---

## Matriz de exposição por camada

| Camada | Rodrigo | Klisman |
|--------|---------|---------|
| `programs/` (on-chain) | **50%** — escrow, deposit, evaluate_trigger, deploy tx | **50%** — policy, errors, payout, pause, smoke txs |
| `oracle-connector/` | S F1 | **Lead** F1 |
| `api/` | S F2 | **Lead** F2, F4 |
| `zk-prover/` | **Lead** F3 | S F3 |
| `web/` | S F3 | **Lead** F3, F4 |
| `docs/` | **Lead** F4 (architecture) | **Lead** F4 (README) |

---

## Regras de colaboração

1. **PR pequeno** — máx. ~300 linhas; review em 24h
2. **Quem faz Primary abre o PR** — Secondary revisa e testa localmente
3. **Branch naming:** `feat/<layer>-<short-desc>` (ex.: `feat/escrow-init`, `feat/payout-execute`)
4. **On-chain PRs:** quem não abriu o PR roda `anchor test` localmente antes de approve
5. **Nunca commitar** keypairs ou `.env`
6. **Sync obrigatório** antes de Fase 2, 3 e 4 — interfaces entre camadas mudam
7. **Pair programming** nas tarefas **S** quando uma camada for nova para quem faz secondary

---

## Ordem de dependências

```
Fase 0 (scaffold + Anchor workspace)  ← J
    ↓
Fase 1: escrow (R) + policy/errors (K)  ←→  oracle (K)     (paralelo)
    ↓
Fase 2: evaluate_trigger (R)  +  execute_payout (K)  →  API (K)
    ↓
Fase 3: ZK proof (R)  →  API /verify (K)  →  dashboard (K)
    ↓
Fase 4: deploy tx (R)  +  smoke txs (K)  →  E2E integration
    ↓
Fase 5: demo
```

---

## Decisão pendente (Fase 0)

Antes de começar, vocês precisam escolher **um** feed Pyth:

| Opção | Prós | Contras |
|-------|------|---------|
| **Price feed** (ex.: SOL/USD) | Fácil de simular trigger, dados estáveis | Menos "insurance feel" |
| **Climate feed** | Alinha com use case agrícola do PRD | Feed pode ser menos trivial de testar |

**Recomendação para MVP:** price feed — mais fácil de disparar trigger na demo; depois trocam o feed sem mudar arquitetura.

---

## Infraestrutura (local first)

| Componente | MVP | Deploy obrigatório? |
|------------|-----|---------------------|
| Programas Anchor | Devnet | **Sim** |
| Pyth feeds | Já existem on-chain | Não (consumir apenas) |
| Oracle connector | Local | Não |
| ZK prover | Local | Não |
| API + PostgreSQL | Docker local | Não |
| Dashboard | `npm run dev` | Não |
| AWS | Post-MVP | Não |
