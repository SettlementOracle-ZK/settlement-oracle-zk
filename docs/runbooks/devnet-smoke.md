# Devnet smoke test runbook

Manual E2E on Solana devnet or **local test-validator**. `make devnet-smoke` auto-detects `http://127.0.0.1:8899` when a local validator is running; otherwise it uses public devnet (requires funded wallet `~/.config/solana/id.json`).

## Program

| Field | Value |
|-------|--------|
| Program ID | `987M3ZdtXNuZu7jfA1TtTHNgYThNHEYyGVP5sq42j1Rd` |
| Cluster | devnet |
| Pyth SOL/USD feed | `7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE` |

## Build + deploy (Rodrigo 4.1)

```bash
solana config set --url devnet
solana airdrop 2
PATH="$HOME/.cargo/bin:$PATH" anchor build --ignore-keys
PATH="$HOME/.cargo/bin:$PATH" anchor deploy --provider.cluster devnet --ignore-keys
solana program show 987M3ZdtXNuZu7jfA1TtTHNgYThNHEYyGVP5sq42j1Rd   # Klisman 4.7
```

## Smoke part 1 (Rodrigo 4.2)

1. `initialize_policy` — policy id, holder, expiry, asset_class
2. `initialize_escrow` — link to policy, set `trigger_threshold`
3. `deposit_premium` — fund escrow PDA

Assert escrow account `status = Active`, `amount > 0`.

## Smoke part 2 (Klisman 4.6)

1. `evaluate_trigger` — pass Pyth price feed account (`PYTH_PRICE_FEED`)
2. Assert escrow `status = Triggered`
3. `execute_payout` — assert holder received lamports, `status = Paid`

```bash
npm run evaluate-trigger --prefix scripts -- --policy-id <hex> --send
```

## Off-chain index

After on-chain payout, register proof + settlement via `make settlement-flow` or update DB rows with tx signature.

## Verify deploy

```bash
solana program show 987M3ZdtXNuZu7jfA1TtTHNgYThNHEYyGVP5sq42j1Rd
anchor test
make test-all
```
