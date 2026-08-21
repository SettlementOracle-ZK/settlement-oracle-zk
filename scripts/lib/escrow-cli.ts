import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { AnchorProvider, Program, Wallet, type Idl } from "@coral-xyz/anchor";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  type TransactionSignature,
} from "@solana/web3.js";
import { mockPythPda } from "../mock-pyth-pda.js";

export const POLICY_SEED = Buffer.from("policy");
export const ESCROW_SEED = Buffer.from("escrow");
export const DEFAULT_PROGRAM_ID = "DqdgQv57RWZ4RVUQ7v6SMvTfA11gFph2TNKZDz7sa3Ap";
export const DEVNET_RPC_URL = "https://api.devnet.solana.com";

export type EscrowIdl = Idl & { address?: string };

export type PolicyPdAs = {
  policyId: Buffer;
  policyIdHex: string;
  policyPda: PublicKey;
  escrowPda: PublicKey;
};

export function repoRoot(): string {
  return path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
}

export function loadIdl(): EscrowIdl {
  const idlPath = path.join(repoRoot(), "target", "idl", "escrow.json");
  if (!fs.existsSync(idlPath)) {
    throw new Error(
      `IDL missing at ${idlPath}. Run \`PATH="$HOME/.cargo/bin:$PATH" anchor build --ignore-keys\`.`,
    );
  }
  return JSON.parse(fs.readFileSync(idlPath, "utf8")) as EscrowIdl;
}

export function loadKeypair(): Keypair {
  const keypairPath =
    process.env.ANCHOR_WALLET ?? path.join(os.homedir(), ".config", "solana", "id.json");
  const secret = JSON.parse(fs.readFileSync(keypairPath, "utf8")) as number[];
  return Keypair.fromSecretKey(Uint8Array.from(secret));
}

export function parsePolicyId(hexStr: string): Buffer {
  const raw = hexStr.startsWith("0x") ? hexStr.slice(2) : hexStr;
  if (!/^[0-9a-fA-F]{64}$/.test(raw)) {
    throw new Error(`invalid policy id: expected 32-byte hex, got ${hexStr}`);
  }
  return Buffer.from(raw, "hex");
}

export function resolveProgramId(idl: EscrowIdl): PublicKey {
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

export function resolveRpcUrl(): string {
  return process.env.SOLANA_RPC_URL ?? DEVNET_RPC_URL;
}

export function isLocalRpc(rpcUrl: string): boolean {
  return rpcUrl.includes("127.0.0.1") || rpcUrl.includes("localhost");
}

export function resolvePriceFeed(programId: PublicKey, rpcUrl: string): PublicKey {
  if (process.env.PYTH_PRICE_FEED) {
    return new PublicKey(process.env.PYTH_PRICE_FEED);
  }
  if (isLocalRpc(rpcUrl)) {
    return new PublicKey("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
  }
  return mockPythPda(programId);
}

export function pdas(programId: PublicKey, policyId: Buffer): PolicyPdAs {
  const [policyPda] = PublicKey.findProgramAddressSync([POLICY_SEED, policyId], programId);
  const [escrowPda] = PublicKey.findProgramAddressSync([ESCROW_SEED, policyId], programId);
  return {
    policyId,
    policyIdHex: policyId.toString("hex"),
    policyPda,
    escrowPda,
  };
}

export function getProgram(connection: Connection, payer: Keypair, idl: EscrowIdl, programId: PublicKey) {
  const wallet = new Wallet(payer);
  const provider = new AnchorProvider(connection, wallet, { commitment: "confirmed" });
  return new Program({ ...(idl as Idl), address: programId.toBase58() }, provider);
}

export async function ensureMockOracle(
  program: Program,
  payer: Keypair,
  priceFeed: PublicKey,
): Promise<TransactionSignature> {
  return program.methods
    .initMockPriceFeed()
    .accounts({
      authority: payer.publicKey,
      priceFeed,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
}

export async function evaluateTriggerForPolicy(
  program: Program,
  payer: Keypair,
  pdas: PolicyPdAs,
  priceFeed: PublicKey,
): Promise<TransactionSignature> {
  return program.methods
    .evaluateTrigger()
    .accounts({
      authority: payer.publicKey,
      escrow: pdas.escrowPda,
      policy: pdas.policyPda,
      priceFeed,
    })
    .rpc();
}

export async function executePayoutForPolicy(
  program: Program,
  pdas: PolicyPdAs,
  holder: PublicKey,
): Promise<TransactionSignature> {
  return program.methods
    .executePayout()
    .accounts({
      escrow: pdas.escrowPda,
      policy: pdas.policyPda,
      holder,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
}
