/**
 * One-shot demo: on-chain payout + off-chain TRIGGERED/PAID index for the dashboard.
 *
 * Modes:
 *   A) Fresh smoke policy (default): runs devnet-smoke E2E, then indexes TRIGGERED + PAID.
 *   B) Browser policy: pass POLICY_ID=<64-hex> (PDAs/holder auto-fetched from chain).
 *
 * Usage (repo root):
 *   make demo-settle
 *   POLICY_ID=<hex> make demo-settle
 *
 * Requires: API at APP_ENV=development (http://127.0.0.1:3000/health)
 */

import { spawn } from "node:child_process";
import path from "node:path";
import { Connection, PublicKey } from "@solana/web3.js";
import {
  ensureMockOracle,
  evaluateTriggerForPolicy,
  executePayoutForPolicy,
  getProgram,
  loadIdl,
  loadKeypair,
  parsePolicyId,
  pdas,
  repoRoot,
  resolvePriceFeed,
  resolveProgramId,
  resolveRpcUrl,
  isLocalRpc,
} from "./lib/escrow-cli.js";
import { mockPythPda } from "./mock-pyth-pda.js";

const API_BASE = (process.env.API_PUBLIC_BASE_URL ?? "http://127.0.0.1:3000").replace(/\/$/, "");
const ASSET_CLASS = process.env.ASSET_CLASS ?? "flight_delay";

export type DemoPolicy = {
  policy_id: string;
  policy_pda: string;
  escrow_pda: string;
  holder: string;
  payout_tx: string;
  payout_amount: number;
  asset_class: string;
};

async function ensureApiUp(): Promise<void> {
  const response = await fetch(`${API_BASE}/health`);
  if (!response.ok) {
    throw new Error(
      `API not reachable at ${API_BASE}. Start: APP_ENV=development cargo run --manifest-path api/Cargo.toml`,
    );
  }
}

function hintProofInvalid(body: string): string {
  if (!body.includes("invalid proof witness")) {
    return body;
  }
  return [
    body,
    "",
    "Likely cause: API binary is stale (proof hash mismatch with zk-prover).",
    "Fix: stop the running API process and restart:",
    "  APP_ENV=development cargo run --manifest-path api/Cargo.toml",
    "Then rerun: make demo-settle",
  ].join("\n");
}

function runCommand(cmd: string, args: string[], env: NodeJS.ProcessEnv): Promise<string> {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, {
      cwd: repoRoot(),
      env: { ...process.env, ...env },
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk: Buffer) => {
      const text = chunk.toString();
      stdout += text;
      process.stdout.write(text);
    });
    child.stderr.on("data", (chunk: Buffer) => {
      const text = chunk.toString();
      stderr += text;
      process.stderr.write(text);
    });
    child.on("close", (code) => {
      if (code !== 0) {
        const detail = hintProofInvalid(`${stderr}\n${stdout}`);
        reject(new Error(`${cmd} ${args.join(" ")} failed (${code}): ${detail}`));
        return;
      }
      resolve(stdout);
    });
  });
}

function parseDemoResultJson(stdout: string): DemoPolicy {
  const line = stdout
    .split("\n")
    .map((l) => l.trim())
    .find((l) => l.startsWith("DEMO_RESULT_JSON="));
  if (!line) {
    throw new Error("devnet-smoke did not emit DEMO_RESULT_JSON= (update scripts/devnet-smoke.ts)");
  }
  return JSON.parse(line.slice("DEMO_RESULT_JSON=".length)) as DemoPolicy;
}

async function settleExistingPolicy(policyIdHex: string): Promise<DemoPolicy> {
  const policyId = parsePolicyId(policyIdHex);
  const idl = loadIdl();
  const programId = resolveProgramId(idl);
  const rpcUrl = resolveRpcUrl();
  const address = pdas(programId, policyId);
  const priceFeed = resolvePriceFeed(programId, rpcUrl);
  const payer = loadKeypair();
  const connection = new Connection(rpcUrl, "confirmed");
  const program = getProgram(connection, payer, idl, programId);

  const policy = await program.account.policyAccount.fetch(address.policyPda);
  const escrowBefore = await program.account.escrowAccount.fetch(address.escrowPda);
  const holder = policy.holder as PublicKey;
  const payoutAmount = Number(escrowBefore.amount.toString());
  const status = JSON.stringify(escrowBefore.status);

  console.log("\n=== Existing policy on-chain ===");
  console.log("policy_id ", address.policyIdHex);
  console.log("policy_pda", address.policyPda.toBase58());
  console.log("escrow_pda", address.escrowPda.toBase58());
  console.log("holder    ", holder.toBase58());
  console.log("escrow    ", status, payoutAmount, "lamports");

  const usingMock =
    !process.env.PYTH_PRICE_FEED && !isLocalRpc(rpcUrl) && priceFeed.equals(mockPythPda(programId));
  if (usingMock) {
    console.log("\n[1/3] refresh mock oracle (150 min delay)");
    const sig = await ensureMockOracle(program, payer, priceFeed);
    console.log("  sig", sig);
  }

  if (!status.toLowerCase().includes("triggered") && !status.toLowerCase().includes("paid")) {
    console.log("\n[2/3] evaluate_trigger");
    const triggerSig = await evaluateTriggerForPolicy(program, payer, address, priceFeed);
    console.log("  sig", triggerSig);
  } else {
    console.log("\n[2/3] evaluate_trigger skipped (already triggered/paid)");
  }

  let payoutTx: string;
  const escrowMid = await program.account.escrowAccount.fetch(address.escrowPda);
  const midStatus = JSON.stringify(escrowMid.status);
  if (midStatus.toLowerCase().includes("paid")) {
    console.log("\n[3/3] execute_payout skipped (already paid)");
    payoutTx = process.env.PAYOUT_TX ?? "";
    if (!payoutTx) {
      throw new Error("Escrow already Paid — set PAYOUT_TX to the execute_payout signature for API index.");
    }
  } else {
    console.log("\n[3/3] execute_payout");
    payoutTx = await executePayoutForPolicy(program, address, holder);
    console.log("  sig", payoutTx);
  }

  return {
    policy_id: address.policyIdHex,
    policy_pda: address.policyPda.toBase58(),
    escrow_pda: address.escrowPda.toBase58(),
    holder: holder.toBase58(),
    payout_tx: payoutTx,
    payout_amount: payoutAmount,
    asset_class: ASSET_CLASS,
  };
}

async function runSettlementFlow(policy: DemoPolicy): Promise<{ proof_hash: string; settlement_id: string }> {
  console.log("\n=== Off-chain TRIGGERED + ZK ===");
  const stdout = await runCommand("npm", ["run", "settlement-flow", "--prefix", "scripts"], {
    POLICY_ID: policy.policy_id,
    POLICY_PDA: policy.policy_pda,
    ESCROW_PDA: policy.escrow_pda,
    POLICY_HOLDER: policy.holder,
    ASSET_CLASS: policy.asset_class,
    SETTLEMENT_STATUS: "TRIGGERED",
    API_PUBLIC_BASE_URL: API_BASE,
  });

  const proofLine = stdout.split("\n").find((l) => l.startsWith("proof_hash "));
  const settlementLine = stdout.split("\n").find((l) => l.startsWith("settlement_id "));
  if (!proofLine || !settlementLine) {
    throw new Error("settlement-flow output missing proof_hash or settlement_id");
  }
  return {
    proof_hash: proofLine.slice("proof_hash ".length).trim(),
    settlement_id: settlementLine.slice("settlement_id ".length).trim(),
  };
}

async function registerPaid(
  policy: DemoPolicy,
  proofHash: string,
): Promise<{ id: string; verification_url?: string }> {
  console.log("\n=== Index PAID ===");
  const response = await fetch(`${API_BASE}/settlements/register`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      policy_id: policy.policy_id,
      status: "PAID",
      proof_hash: proofHash,
      holder: policy.holder,
      asset_class: policy.asset_class,
      policy_pda: policy.policy_pda,
      escrow_pda: policy.escrow_pda,
      tx_signature: policy.payout_tx,
      payout_amount: policy.payout_amount,
    }),
  });
  if (!response.ok) {
    throw new Error(`${response.status} /settlements/register: ${await response.text()}`);
  }
  return response.json() as Promise<{ id: string; verification_url?: string }>;
}

async function main(): Promise<void> {
  await ensureApiUp();

  const policyIdEnv = process.env.POLICY_ID?.replace(/^0x/, "");
  let policy: DemoPolicy;

  if (policyIdEnv) {
    policy = await settleExistingPolicy(policyIdEnv);
  } else {
    console.log("=== Fresh devnet smoke (new policy) ===");
    const stdout = await runCommand("npm", ["run", "devnet-smoke", "--prefix", "scripts"], {});
    policy = parseDemoResultJson(stdout);
  }

  const { proof_hash, settlement_id } = await runSettlementFlow(policy);
  const paid = await registerPaid(policy, proof_hash);

  console.log("\n=== Demo complete ===");
  console.log("policy_id      ", policy.policy_id);
  console.log("holder         ", policy.holder);
  console.log("proof_hash     ", proof_hash);
  console.log("settlement_id  ", settlement_id);
  console.log("paid_index_id  ", paid.id);
  console.log("payout_tx      ", policy.payout_tx);
  console.log("verify         ", `${API_BASE}/verify/${proof_hash}`);
  console.log("explorer       ", `https://explorer.solana.com/tx/${policy.payout_tx}?cluster=devnet`);
  console.log("dashboard      ", "http://localhost:3001/explorer");
  console.log("\nDEMO_SUMMARY_JSON=" + JSON.stringify({ policy, proof_hash, settlement_id, paid_id: paid.id }));
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
