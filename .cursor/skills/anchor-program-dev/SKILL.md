---
name: anchor-program-dev
description: >-
  Scaffold, develop, and test Anchor programs for SettlementOracle ZK escrow and
  payout logic. Use when creating or modifying Solana programs, writing Anchor
  instructions, defining account structs, or running anchor test.
---

# Anchor Program Development

## Prerequisites
- Solana CLI installed (`solana --version`)
- Anchor CLI installed (`anchor --version`)
- Local validator or devnet configured

## Workflow

```
Task Progress:
- [ ] Step 1: Scaffold or locate program
- [ ] Step 2: Define account structs
- [ ] Step 3: Implement instructions
- [ ] Step 4: Write integration tests
- [ ] Step 5: Run anchor test
```

**Step 1: Scaffold**
```bash
anchor init programs/escrow --template multiple
```

**Step 2: Define accounts**
- `EscrowAccount`: policy_id, authority, amount, trigger_threshold, status, bump
- `PolicyAccount`: holder, expiry, asset_class, created_at

**Step 3: Implement instructions**
- `initialize_escrow` — create PDA vault, deposit premium
- `evaluate_trigger` — read oracle CPI, check condition
- `execute_payout` — transfer from vault to holder (requires trigger = true)
- `pause` / `unpause` — circuit breaker (authority only)

**Step 4: Integration tests**
- Test happy path: deposit → trigger → payout
- Test rejection: stale oracle → no payout
- Test circuit breaker: pause → payout fails

**Step 5: Verify**
```bash
anchor test
```

## Additional Resources
- See `.cursor/rules/anchor-programs.mdc` for security checklist
