/**
 * Devnet E2E smoke (runbook 4.2 + 4.6):
 * initialize_policy → initialize_escrow → deposit_premium → evaluate_trigger → execute_payout
 *
 * Usage (repo root):
 *   npm install --prefix scripts
 *   PATH="$HOME/.cargo/bin:$PATH" anchor build --ignore-keys
 *   npm run devnet-smoke --prefix scripts
 *
 * Env: SOLANA_RPC_URL, ANCHOR_WALLET, ESCROW_PROGRAM_ID, PYTH_PRICE_FEED,
 *      SMOKE_DEPOSIT_LAMPORTS (default 10000 = 0.00001 SOL)
 */

import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { AnchorProvider, Program, Wallet, type Idl } from "@coral-xyz/anchor";
import BN from "bn.js";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import { isInitializedMockFeed, mockPythPda } from "./mock-pyth-pda.js";

const POLICY_SEED = Buffer.from("policy");
const ESCROW_SEED = Buffer.from("escrow");
const DEFAULT_PROGRAM_ID = "987M3ZdtXNuZu7jfA1TtTHNgYThNHEYyGVP5sq42j1Rd";
const DEFAULT_PYTH_FEED = "7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE";
const LOCAL_VALIDATOR_URL = "http://127.0.0.1:8899";
const DEVNET_RPC_URL = "https://api.devnet.solana.com";

const TRIGGER_THRESHOLD = new BN(process.env.TRIGGER_THRESHOLD_MINUTES ?? "120");
/** Default smoke deposit: 10_000 lamports (0.00001 SOL). Override with SMOKE_DEPOSIT_LAMPORTS. */
const DEFAULT_DEPOSIT_LAMPORTS = 10_000;
const POLICY_EXPIRY = new BN("4102444800");
const ASSET_CLASS = Buffer.alloc(32);
Buffer.from("flight_delay").copy(ASSET_CLASS);

type EscrowIdl = Idl & { address?: string };

function repoRoot(): string {
  return path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
}

function loadIdl(): EscrowIdl {
  const idlPath = path.join(repoRoot(), "target", "idl", "escrow.json");
  if (!fs.existsSync(idlPath)) {
    throw new Error(
      `IDL missing at ${idlPath}. Run \`PATH="$HOME/.cargo/bin:$PATH" anchor build --ignore-keys\`.`,
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

function statusLabel(status: unknown): string {
  if (typeof status === "object" && status !== null) {
    return Object.keys(status as Record<string, unknown>)[0] ?? "unknown";
  }
  return String(status);
}

function parseDepositLamports(): BN {
  const raw = process.env.SMOKE_DEPOSIT_LAMPORTS ?? String(DEFAULT_DEPOSIT_LAMPORTS);
  const value = Number(raw);
  if (!Number.isFinite(value) || value < 1 || !Number.isInteger(value)) {
    throw new Error(
      `SMOKE_DEPOSIT_LAMPORTS must be a positive integer (lamports), got "${raw}"`,
    );
  }
  return new BN(value);
}

function formatSol(lamports: number): string {
  return `${(lamports / LAMPORTS_PER_SOL).toFixed(9)} SOL`;
}

function policyIdHex(policyId: Buffer): string {
  return policyId.toString("hex");
}

function isLocalRpc(rpcUrl: string): boolean {
  return rpcUrl.includes("127.0.0.1") || rpcUrl.includes("localhost");
}

async function ensureMockOracle(
  program: Program,
  connection: Connection,
  payer: Keypair,
  programId: PublicKey,
  priceFeed: PublicKey,
): Promise<void> {
  const existing = await connection.getAccountInfo(priceFeed);
  if (isInitializedMockFeed(existing?.data)) {
    console.log("mock oracle already initialized — re-init to refresh timestamp");
  } else {
    console.log("mock oracle missing — init (one-time rent on devnet)");
  }
  const sig = await program.methods
    .initMockPriceFeed()
    .accounts({
      authority: payer.publicKey,
      priceFeed,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
  console.log("  init sig", sig);
}

async function resolvePriceFeed(programId: PublicKey, rpcUrl: string): Promise<PublicKey> {
  if (process.env.PYTH_PRICE_FEED) {
    return new PublicKey(process.env.PYTH_PRICE_FEED);
  }
  if (isLocalRpc(rpcUrl)) {
    return new PublicKey(DEFAULT_PYTH_FEED);
  }
  return mockPythPda(programId);
}

function pdas(programId: PublicKey, policyId: Buffer) {
  const [policy] = PublicKey.findProgramAddressSync(
    [POLICY_SEED, policyId],
    programId,
  );
  const [escrow] = PublicKey.findProgramAddressSync(
    [ESCROW_SEED, policyId],
    programId,
  );
  return { policy, escrow };
}

function resolveProgramId(idl: EscrowIdl): PublicKey {
  if (process.env.ESCROW_PROGRAM_ID) {
    return new PublicKey(process.env.ESCROW_PROGRAM_ID);
  }
  const keypairPath = path.join(repoRoot(), "target", "deploy", "escrow-keypair.json");
  if (fs.existsSync(keypairPath)) {
    const secret = JSON.parse(fs.readFileSync(keypairPath, "utf8")) as number[];
    return Keypair.fromSecretKey(Uint8Array.from(secret)).publicKey;
  }
  return new PublicKey(idl.address ?? DEFAULT_PROGRAM_ID);
}

async function resolveRpcUrl(): Promise<string> {
  if (process.env.SOLANA_RPC_URL) {
    return process.env.SOLANA_RPC_URL;
  }
  try {
    const local = new Connection(LOCAL_VALIDATOR_URL, "confirmed");
    await local.getVersion();
    console.log(`Local validator detected — using ${LOCAL_VALIDATOR_URL}`);
    return LOCAL_VALIDATOR_URL;
  } catch {
    console.log(`No local validator — using ${DEVNET_RPC_URL}`);
    return DEVNET_RPC_URL;
  }
}

function lowBalanceHint(rpcUrl: string, balance: number): string {
  const isLocal =
    rpcUrl.includes("127.0.0.1") || rpcUrl.includes("localhost");
  if (isLocal) {
    return `Deployer balance too low (${balance} lamports). Run: solana airdrop 10 --url ${rpcUrl}`;
  }
  return [
    `Deployer balance too low (${balance} lamports) on ${rpcUrl}.`,
    "Start a local validator (see docs/runbooks/run-locally.md) and rerun make devnet-smoke,",
    "or fund the wallet: solana airdrop 2 --url devnet",
  ].join(" ");
}

async function main(): Promise<void> {
  const depositLamports = parseDepositLamports();
  const idl = loadIdl();
  const programId = resolveProgramId(idl);
  const rpcUrl = await resolveRpcUrl();
  const priceFeed = await resolvePriceFeed(programId, rpcUrl);

  const payer = loadKeypair();
  const holder = Keypair.generate();
  const policyId = crypto.randomBytes(32);
  const { policy, escrow } = pdas(programId, policyId);

  const connection = new Connection(rpcUrl, "confirmed");
  const wallet = new Wallet(payer);
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  const program = new Program({ ...(idl as Idl), address: programId.toBase58() }, provider);

  const usingProgramMock =
    !process.env.PYTH_PRICE_FEED && !isLocalRpc(rpcUrl) && priceFeed.equals(mockPythPda(programId));
  if (usingProgramMock) {
    console.log("\n[0/5] bootstrap mock oracle (devnet)");
    await ensureMockOracle(program, connection, payer, programId, priceFeed);
  }

  const balance = await connection.getBalance(payer.publicKey);
  // Rent for 2 PDAs + ~5 tx fees; deposit is returned to holder on payout (minus rent left on PDAs).
  const minBalance = depositLamports.toNumber() + 5_000_000;
  if (balance < minBalance) {
    throw new Error(
      `${lowBalanceHint(rpcUrl, balance)} (need ~${formatSol(minBalance)} including deposit + rent/fees buffer)`,
    );
  }

  console.log("=== Devnet smoke ===");
  console.log("rpc       ", rpcUrl);
  console.log("deposit   ", depositLamports.toString(), "lamports", `(${formatSol(depositLamports.toNumber())})`);
  console.log("program   ", programId.toBase58());
  console.log("payer     ", payer.publicKey.toBase58());
  console.log("holder    ", holder.publicKey.toBase58());
  console.log("policyId  ", policyIdHex(policyId));
  console.log("policy PDA", policy.toBase58());
  console.log("escrow PDA", escrow.toBase58());
  console.log("pyth feed ", priceFeed.toBase58());

  const holderRent = await connection.getMinimumBalanceForRentExemption(0);
  let holderBefore = await connection.getBalance(holder.publicKey);
  if (holderBefore < holderRent) {
    const fundSig = await connection.sendTransaction(
      new Transaction().add(
        SystemProgram.transfer({
          fromPubkey: payer.publicKey,
          toPubkey: holder.publicKey,
          lamports: holderRent - holderBefore,
        }),
      ),
      [payer],
    );
    await connection.confirmTransaction(fundSig, "confirmed");
    holderBefore = await connection.getBalance(holder.publicKey);
    console.log("holder rent prefund", holderRent, "lamports");
  }

  console.log("\n[1/5] initialize_policy");
  const sig1 = await program.methods
    .initializePolicy(
      [...policyId],
      holder.publicKey,
      POLICY_EXPIRY,
      [...ASSET_CLASS],
    )
    .accounts({
      authority: payer.publicKey,
      policy,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
  console.log("  sig", sig1);

  console.log("[2/5] initialize_escrow");
  const sig2 = await program.methods
    .initializeEscrow([...policyId], TRIGGER_THRESHOLD)
    .accounts({
      authority: payer.publicKey,
      policy,
      escrow,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
  console.log("  sig", sig2);

  console.log("[3/5] deposit_premium");
  const sig3 = await program.methods
    .depositPremium(depositLamports)
    .accounts({
      authority: payer.publicKey,
      escrow,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
  console.log("  sig", sig3);

  const afterDeposit = await program.account.escrowAccount.fetch(escrow);
  if (statusLabel(afterDeposit.status) !== "active") {
    throw new Error(`expected Active after deposit, got ${statusLabel(afterDeposit.status)}`);
  }
  if (!afterDeposit.amount.eq(depositLamports)) {
    throw new Error(
      `expected amount ${depositLamports}, got ${afterDeposit.amount.toString()}`,
    );
  }
  console.log("  escrow status Active, amount", afterDeposit.amount.toString());

  console.log("[4/5] evaluate_trigger");
  const sig4 = await program.methods
    .evaluateTrigger()
    .accounts({
      authority: payer.publicKey,
      escrow,
      policy,
      priceFeed,
    })
    .rpc();
  console.log("  sig", sig4);

  const afterTrigger = await program.account.escrowAccount.fetch(escrow);
  if (statusLabel(afterTrigger.status) !== "triggered") {
    throw new Error(`expected Triggered, got ${statusLabel(afterTrigger.status)}`);
  }
  console.log("  escrow status Triggered");

  console.log("[5/5] execute_payout");
  const sig5 = await program.methods
    .executePayout()
    .accounts({
      escrow,
      policy,
      holder: holder.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
  console.log("  sig", sig5);

  const afterPayout = await program.account.escrowAccount.fetch(escrow);
  if (statusLabel(afterPayout.status) !== "paid") {
    throw new Error(`expected Paid, got ${statusLabel(afterPayout.status)}`);
  }
  if (!afterPayout.amount.isZero()) {
    throw new Error(`expected amount 0 after payout, got ${afterPayout.amount.toString()}`);
  }

  const holderAfter = await connection.getBalance(holder.publicKey);
  const received = holderAfter - holderBefore;
  if (received !== depositLamports.toNumber()) {
    throw new Error(
      `holder received ${received} lamports, expected ${depositLamports.toString()}`,
    );
  }

  console.log("\n=== Smoke PASSED ===");
  console.log("deposit spent (returned to holder)", formatSol(received));
  console.log("policy-id for scripts:", policyIdHex(policyId));
  console.log("last tx:", sig5);
  console.log(
    "DEMO_RESULT_JSON=" +
      JSON.stringify({
        policy_id: policyIdHex(policyId),
        policy_pda: policy.toBase58(),
        escrow_pda: escrow.toBase58(),
        holder: holder.publicKey.toBase58(),
        payout_tx: sig5,
        payout_amount: depositLamports.toNumber(),
        asset_class: "flight_delay",
      }),
  );
}

main().catch((err: unknown) => {
  const msg = err instanceof Error ? err.message : String(err);
  if (msg.includes("OracleStale")) {
    console.error(
      [
        "OracleStale: price feed is stale or wrong format.",
        "On devnet, omit PYTH_PRICE_FEED to use the program mock PDA (make devnet-setup first).",
        "On local validator, use: make local-smoke",
      ].join("\n"),
    );
  } else if (msg.includes("Attempt to load a program that does not exist")) {
    console.error(
      [
        "Escrow program is not deployed on this cluster.",
        "Devnet (once): make deploy-devnet && make devnet-oracle",
        "Local: make deploy-local or make local-smoke",
      ].join("\n"),
    );
  } else {
    console.error(msg);
  }
  process.exit(1);
});
