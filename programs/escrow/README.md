# Escrow Program

Anchor program for SettlementOracle ZK MVP — policy registration, escrow vault, premium deposits, circuit breaker, and automated payout.

## Instructions

| Instruction | Owner (plan) | Description |
|-------------|--------------|-------------|
| `initialize_policy` | Klisman | Create `PolicyAccount` PDA |
| `initialize_escrow` | Rodrigo | Create `EscrowAccount` PDA linked to policy |
| `deposit_premium` | Rodrigo | Transfer lamports into escrow vault |
| `pause` / `unpause` | Klisman | Authority-only circuit breaker (`escrow.paused`) |
| `execute_payout` | Klisman | Permissionless crank: transfer premium to holder when `status == Triggered` and not paused |
| `evaluate_trigger` | Rodrigo (2.1) | Read oracle feed on-chain; set `Triggered` when **delay >= threshold** (minutes); fail closed on stale / low-confidence / paused |

## Accounts

- **PolicyAccount** — `policy_id`, `holder`, `expiry`, `asset_class`, `created_at`, `bump`
- **EscrowAccount** — `policy_id`, `authority`, `amount`, `trigger_threshold`, `status`, `paused`, `bump`

`execute_payout` moves the tracked premium with `sub_lamports` / `add_lamports` (the escrow PDA is program-owned, so `system_program::transfer` cannot debit it). Rent-exempt lamports stay on the PDA. Status becomes `Paid` and `amount` is zeroed.

## PDA seeds

- Policy: `[b"policy", policy_id]`
- Escrow: `[b"escrow", policy_id]`

## Interface contract for `evaluate_trigger` (Rodrigo 2.1)

- Instruction name: `evaluate_trigger`
- Accounts: `authority` (signer / fee payer), `escrow` (mut PDA), `policy` (PDA), `price_feed` (Pyth legacy price account)
- On success when the condition is met: `escrow.status = Triggered`
- Trigger rule: mock oracle reports **delay minutes**; payout when `delay >= escrow.trigger_threshold` (default 120 min)
- Fail closed: `OracleStale` / `OracleLowConfidence` / `Paused` / `TriggerNotMet`
- Must **not** transfer funds — payout stays in `execute_payout`

Client: [`scripts/submit-evaluate-trigger.ts`](../../scripts/submit-evaluate-trigger.ts) (`npm run evaluate-trigger --prefix scripts -- --policy-id <hex>`). Pass `PYTH_PRICE_FEED` (Solana account pubkey) for the SOL/USD feed.

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
