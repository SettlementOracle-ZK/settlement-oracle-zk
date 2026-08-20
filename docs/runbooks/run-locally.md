# Run locally — SettlementOracle ZK MVP

End-to-end local stack: PostgreSQL, API, dashboard, oracle + ZK flow.

## Prerequisites

- Docker (PostgreSQL)
- Rust 1.89 + `~/.cargo/bin` on PATH (for `cargo-build-sbf` / Anchor)
- Anchor CLI 1.1.x, Solana CLI
- Node.js 20+

## 1. Environment

```bash
cp .env.example .env
cp web/.env.example web/.env.local
```

## 2. Database + API

**Requires Docker Desktop running.**

```bash
make db-up          # starts Postgres on :5433
make db-migrate
cargo run --manifest-path api/Cargo.toml   # or with APP_ENV=development explicit
```

If `make db-migrate` fails with *pool timed out*, Postgres is not up — open Docker Desktop first.

API: http://127.0.0.1:3000 — `GET /health`, `/policies/:id`, `/settlements/:id`, `/verify/:hash`

## 3. Dashboard

```bash
cd web && npm install && npm run dev
```

Open http://localhost:3001 — Policies, Settlement Explorer, Trigger Monitor.

## 4. Oracle + ZK off-chain flow

```bash
cd oracle-connector && npm install && npm test
cd ../zk-prover && npm install && npm test
cd .. && npm install --prefix scripts
make settlement-flow   # Pyth → prover → POST /proofs + /settlements/register
```

Requires API running with `APP_ENV=development`.

## 5. On-chain local (sem devnet)

**Recomendado — um comando só** (validador + mock Pyth legacy + deploy + smoke):

```bash
make local-smoke
```

O feed `7UVimff...` na devnet pública é conta **Pyth Receiver** (~134 bytes), incompatível com o parser legacy do escrow. `make local-smoke` instala um mock legacy no validador local.

Ou manualmente em dois terminais:

Terminal 1 — validador com feature-set da devnet (necessário para deploy):

```bash
solana-test-validator --reset --url devnet --clone-feature-set \
  --clone 7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE
```

Terminal 2 — deploy + smoke (logo após subir o validador):

```bash
solana config set --url http://127.0.0.1:8899
make deploy-local    # necessário após --reset do validador
make devnet-smoke    # auto-detecta :8899; faz deploy-local se o programa não existir
```

`make devnet-smoke` detecta automaticamente o validador local em `:8899`. Override: `SOLANA_RPC_URL=... make devnet-smoke`.

Nota: após `--reset` do validador o programa some — use `make deploy-local`. Se `evaluate_trigger` falhar com `OracleStale`, o feed Pyth clonado expirou (~60s) — use `make local-smoke`.

## 6. On-chain devnet (opcional)

```bash
solana config set --url devnet
PATH="$HOME/.cargo/bin:$PATH" anchor build
PATH="$HOME/.cargo/bin:$PATH" anchor deploy --provider.cluster devnet
npm run evaluate-trigger --prefix scripts -- --policy-id <64-hex> --send
```

See [`docs/runbooks/devnet-smoke.md`](devnet-smoke.md).

## 7. Tests

```bash
make test-all
```

## Package map

| Path | Command |
|------|---------|
| `programs/escrow/` | `make test-escrow` |
| `oracle-connector/` | `make test-oracle` |
| `zk-prover/` | `make test-zk` |
| `api/` | `make test-api` |
| `web/` | `cd web && npm run build` |
