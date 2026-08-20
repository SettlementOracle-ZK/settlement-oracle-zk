# On-chain security review (Phase 5.2)

## PDAs

- [x] Policy: `[b"policy", policy_id]`
- [x] Escrow: `[b"escrow", policy_id]`
- [x] Bumps stored on accounts

## Signers

- [x] `deposit_premium` — escrow authority only
- [x] `pause` / `unpause` — escrow authority only
- [x] `evaluate_trigger` — permissionless crank (no fund movement)
- [x] `execute_payout` — permissionless when `Triggered` and not paused

## Fail closed

- [x] Oracle stale / low confidence → no trigger
- [x] Paused escrow → no trigger, no payout
- [x] Payout requires `Triggered` status

## Code hygiene

- [x] No `unwrap()` in program handlers — `?` + `ErrorCode`
- [x] No off-chain fund transfers
- [ ] Formal external audit — post-MVP

## Tests

- [x] `cargo test -p escrow` — phase 1, 2, evaluate_trigger suites
