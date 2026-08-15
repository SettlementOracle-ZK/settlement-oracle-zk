/**
 * Submit `evaluate_trigger` via @solana/web3.js + Anchor IDL (task 2.11).
 *
 * Interface contract for Rodrigo (2.1):
 * - Instruction name: evaluate_trigger
 * - Accounts: authority, escrow (mut PDA), policy (PDA); Pyth accounts TBD
 * - On success when the condition is met: escrow.status = Triggered
 * - Fail closed: OracleStale / OracleLowConfidence / Paused
 * - Must not transfer funds (payout stays in execute_payout)
 *
 * Until 2.1 lands this script derives PDAs, loads target/idl/escrow.json, and
 * exits with a clear error if evaluate_trigger is absent — it does not invent
 * a fake instruction.
 *
 * Usage (from repo root):
 *   npm install --prefix scripts
 *   npm run evaluate-trigger --prefix scripts -- --policy-id <64-hex>
 *   npm run evaluate-trigger --prefix scripts -- --policy-id <64-hex> --send
 *
 * Env: SOLANA_RPC_URL, ANCHOR_WALLET (keypair json path), ESCROW_PROGRAM_ID
 */

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { AnchorProvider, Program, Wallet, type Idl } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";

const POLICY_SEED = Buffer.from("policy");
const ESCROW_SEED = Buffer.from("escrow");
const DEFAULT_PROGRAM_ID = "987M3ZdtXNuZu7jfA1TtTHNgYThNHEYyGVP5sq42j1Rd";

type EscrowIdl = Idl & {
  address?: string;
  instructions: { name: string; accounts: { name: string }[] }[];
};

function repoRoot(): string {
  return path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
}

function parseArgs(argv: string[]): { policyId: string; send: boolean } {
  let policyId = "";
  let send = false;
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--send") {
      send = true;
    } else if (arg === "--policy-id") {
      policyId = argv[i + 1] ?? "";
      i += 1;
    } else if (arg.startsWith("--policy-id=")) {
      policyId = arg.slice("--policy-id=".length);
    }
  }
  if (!policyId) {
    throw new Error("missing --policy-id <64-char hex>");
  }
  return { policyId, send };
}

function parsePolicyId(hexStr: string): Buffer {
  const raw = hexStr.startsWith("0x") ? hexStr.slice(2) : hexStr;
  if (!/^[0-9a-fA-F]{64}$/.test(raw)) {
    throw new Error(`invalid policy id: expected 32-byte hex, got ${hexStr}`);
  }
  return Buffer.from(raw, "hex");
}

function loadIdl(): EscrowIdl {
  const idlPath = path.join(repoRoot(), "target", "idl", "escrow.json");
  if (!fs.existsSync(idlPath)) {
    throw new Error(
      `IDL not found at ${idlPath}. Run \`anchor build --ignore-keys\` from the repo root.`,
    );
  }
  return JSON.parse(fs.readFileSync(idlPath, "utf8")) as EscrowIdl;
}

function loadKeypair(): Keypair {
  const keypairPath =
    process.env.ANCHOR_WALLET ?? path.join(os.homedir(), ".config", "solana", "id.json");
  const secret = JSON.parse(fs.readFileSync(keypairPath, "utf8")) as number[];
  return Keypair.fromSecretKey(Uint8Array.from(secret));
}

function findInstruction(idl: EscrowIdl, name: string) {
  return idl.instructions.find((ix) => ix.name === name);
}

async function main(): Promise<void> {
  const { policyId: policyIdHex, send } = parseArgs(process.argv.slice(2));
  const policyId = parsePolicyId(policyIdHex);
  const idl = loadIdl();
  const programId = new PublicKey(
    process.env.ESCROW_PROGRAM_ID ?? idl.address ?? DEFAULT_PROGRAM_ID,
  );

  const [policyPda] = PublicKey.findProgramAddressSync(
    [POLICY_SEED, policyId],
    programId,
  );
  const [escrowPda] = PublicKey.findProgramAddressSync(
    [ESCROW_SEED, policyId],
    programId,
  );

  console.log("programId", programId.toBase58());
  console.log("policyId ", policyIdHex);
  console.log("policy   ", policyPda.toBase58());
  console.log("escrow   ", escrowPda.toBase58());

  const ix = findInstruction(idl, "evaluate_trigger");
  if (!ix) {
    console.error(
      [
        "",
        "evaluate_trigger is not in the IDL yet (Rodrigo 2.1).",
        "Contract: set escrow.status = Triggered on success; fail closed on",
        "OracleStale / OracleLowConfidence / Paused; do not transfer funds.",
        "Accounts: authority, escrow (mut PDA), policy (PDA); Pyth TBD.",
      ].join("\n"),
    );
    process.exit(2);
  }

  const rpcUrl =
    process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com";
  const payer = loadKeypair();
  const connection = new Connection(rpcUrl, "confirmed");
  const wallet = new Wallet(payer);
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  const program = new Program(idl as Idl, provider);

  const builder = program.methods.evaluateTrigger().accounts({
    authority: payer.publicKey,
    escrow: escrowPda,
    policy: policyPda,
  });

  if (!send) {
    const tx = await builder.transaction();
    console.log(
      "dry-run: built evaluate_trigger transaction " +
        `(${tx.instructions.length} instruction(s)). Pass --send to submit.`,
    );
    return;
  }

  const sig = await builder.rpc();
  console.log("signature", sig);
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
