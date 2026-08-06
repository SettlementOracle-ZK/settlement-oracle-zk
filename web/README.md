# Web Dashboard

Next.js frontend for monitoring parametric policies, settlements, and ZK proof / transaction verification.

## Status

**Scaffold only — not implemented**

## Purpose

- **Settlement panel:** real-time view of active contracts, monitored events, and settlement status
- **Transaction & proof explorer:** verify ZK hash and on-chain confirmation for insureds and auditors
- Connect Solana wallets (e.g. Phantom, Solflare) via Wallet Adapter for signed actions when required

## Responsibilities

| Area | Description |
|------|-------------|
| Dashboard | List/filter policies and settlement statuses |
| Explorer | Display tx signatures, proof hashes, verification links |
| Wallet | Wallet Adapter integration for user-facing Solana actions |
| API client | Consume [`api/`](../api/) endpoints for risk and proof payloads |

## Target stack

- **Framework:** React / Next.js
- **Wallets:** Solana Wallet Adapter (Phantom, Solflare, etc.)
- **Network (MVP):** Solana devnet

## Future layout (when implemented)

```
web/
├── package.json
├── app/ or pages/
├── components/
├── lib/
│   ├── api/
│   └── solana/
└── public/
```

## Non-goals (MVP)

- Full multi-tenant insurer CMS
- Mainnet-only features
- Embedding heavy ZK proving in the browser
- Direct third-party claims system writes

## Related

- Product requirements: [`../docs/PRD.md`](../docs/PRD.md) (Design/UX section)
- Agent conventions: [`../AGENTS.md`](../AGENTS.md)
- Cursor rule: `frontend-dashboard`
- Backend: [`../api/`](../api/)
- On-chain: [`../programs/escrow/`](../programs/escrow/)
