# Escrow Program

Solana Anchor program that holds policy premiums in escrow and executes automated payouts when a parametric trigger is confirmed.

## Status

**Scaffold only — not implemented**

## Purpose

- Create and manage on-chain policy / escrow vault accounts
- Escrow premiums from the insurer (or premium payer)
- Release funds to the insured wallet when oracle-backed trigger conditions are met
- Keep all fund movement on-chain (no off-chain transfers)

## Responsibilities

| Area | Description |
|------|-------------|
| Escrow vault | PDA-backed vault for premiums |
| Policy state | Store strike/threshold params and settlement status |
| Payout | Transfer escrowed funds when trigger is authorized |
| Safety | Fail closed on invalid accounts, unauthorized signers, or unsafe state |

## Target stack

- **Runtime:** Solana
- **Framework:** Anchor (Rust)
- **Network (MVP):** Devnet / testnet only

## Future layout (when implemented)

```
programs/escrow/
├── Cargo.toml
├── Xargo.toml          # if needed by Anchor template
├── src/
│   ├── lib.rs
│   ├── instructions/
│   ├── state/
│   └── errors.rs
└── tests/
```

Exact structure will follow the Anchor workspace conventions set at the repo root when programs are initialized.

## Non-goals (MVP)

- Mainnet deployment
- Multi-asset or multi-chain escrow factories
- Dispute / challenge periods on-chain
- Continuous monitoring or keepers as part of this package

## Related

- Product requirements: [`../../docs/PRD.md`](../../docs/PRD.md)
- Agent conventions: [`../../AGENTS.md`](../../AGENTS.md)
- Cursor rule: `anchor-programs` / skill: `anchor-program-dev`
- Upstream of settlement flow: [`../../oracle-connector/`](../../oracle-connector/)
- Downstream consumers: [`../../zk-prover/`](../../zk-prover/), [`../../api/`](../../api/), [`../../web/`](../../web/)
