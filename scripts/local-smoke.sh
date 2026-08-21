#!/usr/bin/env bash
# Full on-chain local smoke: fresh validator + mock legacy Pyth + deploy + devnet-smoke.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

LOCAL_RPC="${LOCAL_VALIDATOR_URL:-http://127.0.0.1:8899}"
FIXTURE_DIR="$ROOT/scripts/.local"
MOCK_PYTH_JSON="$FIXTURE_DIR/mock-pyth.json"
VALIDATOR_LOG="${TMPDIR:-/tmp}/settlement-local-validator.log"

mkdir -p "$FIXTURE_DIR"
MOCK_PYTH=$(
  cd scripts && npx tsx install-mock-pyth.ts "$MOCK_PYTH_JSON"
)
echo "==> Mock legacy Pyth feed: $MOCK_PYTH"

echo "==> Stopping any existing local validator..."
pkill -f 'solana-test-validator' 2>/dev/null || true
sleep 2

echo "==> Starting validator with mock Pyth account..."
solana-test-validator --reset --url devnet --clone-feature-set \
  --account "$MOCK_PYTH" "$MOCK_PYTH_JSON" >"$VALIDATOR_LOG" 2>&1 &
VALIDATOR_PID=$!

cleanup() {
  if kill -0 "$VALIDATOR_PID" 2>/dev/null; then
    kill "$VALIDATOR_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

echo "==> Waiting for validator at $LOCAL_RPC ..."
for _ in $(seq 1 30); do
  if solana cluster-version --url "$LOCAL_RPC" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
if ! solana cluster-version --url "$LOCAL_RPC" >/dev/null 2>&1; then
  echo "Validator failed to start. Log: $VALIDATOR_LOG"
  exit 1
fi

solana config set --url "$LOCAL_RPC" >/dev/null
solana airdrop 10 >/dev/null 2>&1 || true

echo "==> Deploying escrow program..."
make deploy-local LOCAL_VALIDATOR_URL="$LOCAL_RPC"

echo "==> Running on-chain smoke..."
SOLANA_RPC_URL="$LOCAL_RPC" PYTH_PRICE_FEED="$MOCK_PYTH" \
  SMOKE_DEPOSIT_LAMPORTS="${SMOKE_DEPOSIT_LAMPORTS:-10000}" \
  npm run devnet-smoke --prefix scripts

echo "==> Local smoke PASSED"
