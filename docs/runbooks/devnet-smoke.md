# Devnet smoke test runbook

Manual E2E on Solana **devnet** or **local test-validator**. `make devnet-smoke` auto-detects `http://127.0.0.1:8899` when a local validator is running; otherwise it uses public devnet.

## Budget (2 SOL wallet)

| Step | Cost | Notes |
|------|------|--------|
| `make deploy-devnet` | **~1–1.5 SOL once** | Do **not** redeploy on every test |
| `make devnet-oracle` | ~0.02 SOL once | Rent for mock Pyth PDA (~3 KB) |
| Each `make devnet-smoke` | ~0.003–0.005 SOL | New policy/escrow PDAs + tx fees |
| Deposit in smoke | 10_000 lamports default | Returned to holder on payout |
| `make settlement-flow` | **0 SOL** | Off-chain only |

Demo with visible payout: `SMOKE_DEPOSIT_LAMPORTS=500000000 make devnet-smoke` (0.5 SOL deposit).

## One-time devnet setup

```bash
solana config set --url devnet
# Fund wallet (faucet or devnet-pow) — you need ~2 SOL total
PATH="$HOME/.cargo/bin:$PATH" anchor build --ignore-keys

# Deploy program once + init mock oracle (skips deploy if already on-chain)
make devnet-setup
```

Program ID comes from `target/deploy/escrow-keypair.json` (shown after deploy).

## Cheap smoke (repeat many times)

```bash
# Default deposit 0.00001 SOL — omit PYTH_PRICE_FEED to use program mock PDA
make devnet-smoke

# Or explicitly:
SOLANA_RPC_URL=https://api.devnet.solana.com make devnet-smoke
```

Flow on devnet:

1. Bootstrap mock oracle PDA (`init_mock_price_feed` or `refresh_mock_price_feed`)
2. `initialize_policy` → `initialize_escrow` → `deposit_premium`
3. `evaluate_trigger` (mock feed price 50 < threshold 100)
4. `execute_payout`

## Why mock oracle on devnet?

Devnet account `7UVimff...` is a **Pyth Receiver** account (~134 B), not the legacy layout the escrow program reads. The program-owned PDA (`seed = mock_pyth`) stores a valid legacy layout and stays fresh.

## Local alternative (free SOL)

```bash
make local-smoke        # validator + mock JSON feed + deploy + smoke
make local-smoke-keep   # same, validator stays up
```

## Off-chain index

After on-chain payout, register via `make settlement-flow` or `POST /settlements/register` with the smoke tx signature and proof hash.

## Verify

```bash
solana program show $(solana-keygen pubkey target/deploy/escrow-keypair.json)
make test-all
```
