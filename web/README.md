# Web Dashboard

Next.js desk for monitoring parametric policies, settlements, and ZK proof / transaction verification.

## Status

Phase 3: Settlement Explorer, Trigger Monitor, Phantom/Solflare wallet connect. Active Policies is included so the Phase 3 checkpoint can be exercised before Rodrigo tasks 3.1–3.3.

## Views

| Route | Task | What you see |
|-------|------|----------------|
| `/policies` | checkpoint | Indexed policies (API; optional demo seed) |
| `/explorer` | 3.6 | Tx signature, proof hash, Solana Explorer + `/verify` links |
| `/monitor` | 3.7 | SOL/USD vs threshold barograph; stale / low-confidence warnings |
| header wallet | 3.8 | Phantom and Solflare via Wallet Adapter (devnet) |

The browser **does not** read Solana RPC for policy or settlement state. It polls the API gateway. Wallet RPC is used only for adapter connection.

## Run locally

```bash
# from repo root
cp .env.example .env
make db-up
make db-migrate
make db-seed
cargo run --manifest-path api/Cargo.toml

# dashboard env (Next.js does not read the repo-root .env)
cd web
cp .env.example .env.local
npm install
npm run dev
```

Open [http://localhost:3001](http://localhost:3001). API is `127.0.0.1:3000`; the dashboard uses port **3001**.

Env (see [`web/.env.example`](.env.example)): `NEXT_PUBLIC_API_URL`, `NEXT_PUBLIC_SOLANA_RPC_URL`, `NEXT_PUBLIC_USE_FIXTURES`.

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
