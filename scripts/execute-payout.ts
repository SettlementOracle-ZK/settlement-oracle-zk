/**
 * Submit `execute_payout` for an existing policy (escrow must be Triggered).
 *
 * Usage:
 *   npm run execute-payout --prefix scripts -- --policy-id <64-hex> --send
 *
 * Env: SOLANA_RPC_URL, ANCHOR_WALLET, ESCROW_PROGRAM_ID, PYTH_PRICE_FEED
 */

import { Connection, PublicKey } from "@solana/web3.js";
import {
  evaluateTriggerForPolicy,
  executePayoutForPolicy,
  ensureMockOracle,
  getProgram,
  loadIdl,
  loadKeypair,
  parsePolicyId,
  pdas,
  resolvePriceFeed,
  resolveProgramId,
  resolveRpcUrl,
  isLocalRpc,
} from "./lib/escrow-cli.js";
import { mockPythPda } from "./mock-pyth-pda.js";

function parseArgs(argv: string[]): { policyId: string; send: boolean; skipTrigger: boolean } {
  let policyId = "";
  let send = false;
  let skipTrigger = false;
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--send") send = true;
    else if (arg === "--skip-trigger") skipTrigger = true;
    else if (arg === "--policy-id") {
      policyId = argv[i + 1] ?? "";
      i += 1;
    } else if (arg.startsWith("--policy-id=")) {
      policyId = arg.slice("--policy-id=".length);
    }
  }
  if (!policyId) throw new Error("missing --policy-id <64-char hex>");
  return { policyId, send, skipTrigger };
}

async function main(): Promise<void> {
  const { policyId: policyIdHex, send, skipTrigger } = parseArgs(process.argv.slice(2));
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

  console.log("programId", programId.toBase58());
  console.log("policyId ", address.policyIdHex);
  console.log("policy   ", address.policyPda.toBase58());
  console.log("escrow   ", address.escrowPda.toBase58());
  console.log("holder   ", holder.toBase58());
  console.log("amount   ", payoutAmount, "lamports");
  console.log("status   ", JSON.stringify(escrowBefore.status));

  if (!send) {
    console.log("dry-run: pass --send to refresh oracle (if needed), evaluate_trigger, execute_payout");
    return;
  }

  const usingMock =
    !process.env.PYTH_PRICE_FEED && !isLocalRpc(rpcUrl) && priceFeed.equals(mockPythPda(programId));
  if (usingMock) {
    const sig = await ensureMockOracle(program, payer, priceFeed);
    console.log("mock oracle", sig);
  }

  if (!skipTrigger) {
    const triggerSig = await evaluateTriggerForPolicy(program, payer, address, priceFeed);
    console.log("evaluate_trigger", triggerSig);
  }

  const payoutSig = await executePayoutForPolicy(program, address, holder);
  console.log("execute_payout", payoutSig);
  console.log("payout_amount", payoutAmount);
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
