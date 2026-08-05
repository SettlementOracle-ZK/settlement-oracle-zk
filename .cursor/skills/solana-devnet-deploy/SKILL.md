---
name: solana-devnet-deploy
description: >-
  Deploy SettlementOracle ZK Anchor programs to Solana devnet and run
  end-to-end smoke tests. Use when deploying programs, airdropping devnet SOL,
  initializing escrow, or smoke-testing trigger-to-payout flow on devnet.
---

# Solana Devnet Deploy

## Pre-deploy Checklist
- [ ] `anchor test` passes locally
- [ ] Program keypair NOT in repo (in `.gitignore`)
- [ ] `SOLANA_RPC_URL` points to devnet
- [ ] Deployer wallet has devnet SOL (`solana airdrop 2`)

## Deploy Workflow

```bash
# 1. Build
anchor build

# 2. Deploy to devnet
anchor deploy --provider.cluster devnet

# 3. Verify program ID
solana program show <PROGRAM_ID> --url devnet

# 4. Initialize test escrow
# (use project-specific CLI or script)

# 5. Smoke test
# Trigger evaluation → payout on devnet
```

## Post-deploy
- Record program ID in `docs/architecture/mvp-system-overview.md`
- Do NOT deploy to mainnet — devnet/testnet only for MVP

## Troubleshooting
- `Insufficient funds`: run `solana airdrop 2 --url devnet`
- `Program expired`: redeploy with `anchor deploy --provider.cluster devnet`
