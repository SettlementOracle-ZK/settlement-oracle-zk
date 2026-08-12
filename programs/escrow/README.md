# Escrow Program

Solana Anchor program that holds policy premiums in escrow and executes automated payouts when a parametric trigger is confirmed.

## Status

**In progress** — `PolicyAccount` / `EscrowAccount` state, domain errors, and `create_policy` are implemented. Escrow deposit, trigger evaluation, and payout are not wired yet.

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
- **Framework:** Anchor 1.1 (Rust)
- **Network (MVP):** Devnet / testnet only

## Layout

Workspace root files: `Anchor.toml`, `Cargo.toml`, `rust-toolchain.toml`.

```
programs/escrow/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── constants.rs
│   ├── error.rs
│   ├── state.rs
│   ├── instructions.rs
│   └── instructions/
│       └── create_policy.rs
└── tests/
    └── test_create_policy.rs
```

Build from the repo root with `anchor build`. Tests: `anchor test` (runs `cargo test` per `Anchor.toml`).

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
