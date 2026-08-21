# Web Dashboard

Next.js desk for monitoring parametric policies, settlements, and ZK proof / transaction verification.

## Status

Phase 3: Settlement Explorer, Trigger Monitor, Phantom/Solflare wallet connect. Active Policies is included so the Phase 3 checkpoint can be exercised before Rodrigo tasks 3.1–3.3.

## Views

| Route | Task | What you see |
|-------|------|----------------|
| `/policies/new` | **Option B** | Create real flight-delay cover on devnet (Phantom signs 3 txs) |
| `/policies` | checkpoint | Indexed policies (API; optional demo seed) |
| `/explorer` | 3.6 | Tx signature, proof hash, Solana Explorer + `/verify` links |
| `/monitor` | 3.7 | Delay trigger vs oracle index; stale / low-confidence warnings |
| header wallet | 3.8 | Phantom and Solflare via Wallet Adapter (devnet) |

The browser polls the API for indexed state. Wallet RPC is used for **creating policies** (`/policies/new`) and adapter connection.

After `anchor build`, sync the IDL for the dashboard:

```bash
cp target/idl/escrow.json web/lib/idl/escrow.json
```

Env (see [`web/.env.example`](.env.example)): `NEXT_PUBLIC_API_URL`, `NEXT_PUBLIC_SOLANA_RPC_URL`, `NEXT_PUBLIC_ESCROW_PROGRAM_ID`, `NEXT_PUBLIC_USE_FIXTURES`.

## Run locally

```bash
# from repo root
cp .env.example .env
make db-up
make db-migrate
make db-seed
APP_ENV=development cargo run --manifest-path api/Cargo.toml

# dashboard env (Next.js does not read the repo-root .env)
cd web
cp .env.example .env.local
npm install
npm run dev
```

Open [http://localhost:3001/policies/new](http://localhost:3001/policies/new) with Phantom on **devnet** (~0.01 SOL for rent + premium). API must run with `APP_ENV=development` so `POST /policies/register` is enabled.

With `NEXT_PUBLIC_USE_FIXTURES=false` (default), API failures show empty/error states. Set it to `true` only for layout review; fixtures are labeled as demo data, never as a verified proof.

## Non-goals (MVP)

- Full multi-tenant insurer CMS
- Mainnet-only features
- Embedding heavy ZK proving in the browser
- Direct third-party claims system writes

## Related

- Product requirements: [`../docs/PRD.md`](../docs/PRD.md)
- API: [`../api/`](../api/)
- Cursor rule: `frontend-dashboard`
