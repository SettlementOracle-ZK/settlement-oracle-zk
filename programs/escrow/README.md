# Escrow Program

Anchor program for SettlementOracle ZK MVP — policy registration, escrow vault, and premium deposits.

## Instructions (Phase 1)

| Instruction | Owner (plan) | Description |
|-------------|--------------|-------------|
| `initialize_policy` | Klisman | Create `PolicyAccount` PDA |
| `initialize_escrow` | Rodrigo | Create `EscrowAccount` PDA linked to policy |
| `deposit_premium` | Rodrigo | Transfer lamports into escrow vault |

## Accounts

- **PolicyAccount** — `policy_id`, `holder`, `expiry`, `asset_class`, `created_at`, `bump`
- **EscrowAccount** — `policy_id`, `authority`, `amount`, `trigger_threshold`, `status`, `bump`

## PDA seeds

- Policy: `[b"policy", policy_id]`
- Escrow: `[b"escrow", policy_id]`

## Test

Build the SBF artifact first, then run tests **from the repo root**:

```bash
# From repo root
cargo build-sbf --manifest-path programs/escrow/Cargo.toml --tools-version v1.52
cargo test -p escrow
```

Or via Anchor (also from repo root):

```bash
anchor build
anchor test
```

Tests load `target/deploy/escrow.so` at compile time — run `cargo build-sbf` before `cargo test`.
