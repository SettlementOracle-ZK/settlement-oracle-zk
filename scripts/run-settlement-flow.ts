/**
 * Off-chain settlement flow: Pyth oracle → zk-prover → API index.
 *
 * On-chain steps (initialize / deposit / evaluate_trigger / execute_payout)
 * remain manual or via other scripts — this wires oracle + proof + DB index.
 *
 * Usage (repo root):
 *   npm install --prefix scripts
 *   npm install --prefix oracle-connector
 *   npm install --prefix zk-prover
 *   cp .env.example .env && make db-up && make db-migrate
 *   npm run settlement-flow --prefix scripts
 *
 * Env: API_PUBLIC_BASE_URL, DATABASE_URL (via API), THRESHOLD, ASSET_CLASS
 */

import { evaluateTrigger, PythHermesClient } from "../oracle-connector/src/index.ts";
import { generateProof } from "../zk-prover/src/index.ts";

const API_BASE = (process.env.API_PUBLIC_BASE_URL ?? "http://127.0.0.1:3000").replace(/\/$/, "");
const THRESHOLD = Number(process.env.TRIGGER_THRESHOLD ?? "120");
const ASSET_CLASS = process.env.ASSET_CLASS ?? "flight_delay";
const TRIGGER_OPERATOR = (process.env.TRIGGER_OPERATOR ?? "gte") as "lt" | "lte" | "gt" | "gte";
const POLICY_ID = (process.env.POLICY_ID ?? "aa".repeat(32)).replace(/^0x/, "");

async function ensureApiUp(): Promise<void> {
  try {
    const response = await fetch(`${API_BASE}/health`);
    if (!response.ok) {
      throw new Error(`health returned ${response.status}`);
    }
  } catch {
    throw new Error(
      [
        `API not reachable at ${API_BASE}.`,
        "",
        "1. Start Docker Desktop",
        "2. make db-up && make db-migrate",
        "3. APP_ENV=development cargo run --manifest-path api/Cargo.toml",
        "4. make settlement-flow",
      ].join("\n"),
    );
  }
}

async function postJson<T>(path: string, body: unknown): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`${API_BASE}${path}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
  } catch (err) {
    const hint =
      "API unreachable. Start: make db-up && make db-migrate && APP_ENV=development cargo run --manifest-path api/Cargo.toml";
    const detail = err instanceof Error ? err.message : String(err);
    throw new Error(`${hint} (${detail})`);
  }
  if (!response.ok) {
    const text = await response.text();
    throw new Error(`${response.status} ${path}: ${text}`);
  }
  return response.json() as Promise<T>;
}

async function main(): Promise<void> {
  await ensureApiUp();

  const client = new PythHermesClient();
  const feed = await client.getLatestPriceFeed();
  const trigger = evaluateTrigger(feed, { threshold: THRESHOLD, operator: TRIGGER_OPERATOR });

  console.log("oracle", {
    price: feed.price,
    conf: feed.conf,
    publishTime: feed.publishTime,
    triggered: trigger.triggered,
    reason: trigger.reason,
  });

  const proof = generateProof({
    feedId: feed.feedId,
    oraclePrice: feed.price,
    oracleConf: feed.conf,
    publishTime: feed.publishTime,
    threshold: THRESHOLD,
    operator: TRIGGER_OPERATOR,
    assetClass: ASSET_CLASS,
    verificationBaseUrl: API_BASE,
  });

  console.log("proof_hash", proof.proof_hash);

  await postJson("/proofs", {
    proof_hash: proof.proof_hash,
    asset_class: proof.payload.asset_class,
    risk_score: proof.payload.risk_score,
    scale: proof.payload.scale,
    model_confidence: proof.payload.model_confidence,
    timestamp: proof.payload.timestamp,
    public_inputs: proof.witness,
  });

  const settlement = await postJson<{ id: string; verification_url?: string }>(
    "/settlements/register",
    {
      policy_id: POLICY_ID,
      status:
        process.env.SETTLEMENT_STATUS ??
        (trigger.triggered ? "TRIGGERED" : "PENDING"),
      proof_hash: proof.proof_hash,
      holder: process.env.POLICY_HOLDER ?? "FlowDemoHolder111111111111111111111111",
      asset_class: ASSET_CLASS,
      policy_pda: process.env.POLICY_PDA ?? "PolicyFlowDemo1111111111111111111111",
      escrow_pda: process.env.ESCROW_PDA ?? "EscrowFlowDemo1111111111111111111111",
    },
  );

  console.log("settlement_id", settlement.id);
  console.log("verify", `${API_BASE}/verify/${proof.proof_hash}`);
  console.log("settlement_detail", `${API_BASE}/settlements/${settlement.id}`);
  console.log(
    "\nNext on-chain (devnet): initialize_policy → initialize_escrow → deposit_premium → evaluate_trigger → execute_payout",
  );
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
