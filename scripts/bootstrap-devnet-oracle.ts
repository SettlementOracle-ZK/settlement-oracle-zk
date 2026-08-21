/**
 * Init or refresh the escrow program's mock Pyth PDA on devnet/local.
 *
 * Usage:
 *   npm run bootstrap-devnet-oracle --prefix scripts
 *
 * Env: SOLANA_RPC_URL, ANCHOR_WALLET, ESCROW_PROGRAM_ID
 */

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { AnchorProvider, Program, Wallet, type Idl } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { isInitializedMockFeed, mockPythPda } from "./mock-pyth-pda.js";

const DEFAULT_PROGRAM_ID = "987M3ZdtXNuZu7jfA1TtTHNgYThNHEYyGVP5sq42j1Rd";

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

async function main(): Promise<void> {
  const idl = loadIdl();
  const programId = resolveProgramId(idl);
  const rpcUrl = process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com";
  const payer = loadKeypair();
  const priceFeed = mockPythPda(programId);

  const connection = new Connection(rpcUrl, "confirmed");
  const wallet = new Wallet(payer);
  const provider = new AnchorProvider(connection, wallet, { commitment: "confirmed" });
  const program = new Program({ ...(idl as Idl), address: programId.toBase58() }, provider);

  const existing = await connection.getAccountInfo(priceFeed);
  const initialized = isInitializedMockFeed(existing?.data);

  console.log("=== Bootstrap mock oracle ===");
  console.log("rpc       ", rpcUrl);
  console.log("program   ", programId.toBase58());
  console.log("payer     ", payer.publicKey.toBase58());
  console.log("mock feed ", priceFeed.toBase58());
  console.log("status    ", initialized ? "re-init (refresh)" : "init");

  const sig = await program.methods
    .initMockPriceFeed()
    .accounts({
      authority: payer.publicKey,
      priceFeed,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  console.log("sig       ", sig);
  console.log("\nSet for smoke:");
  console.log(`  export PYTH_PRICE_FEED=${priceFeed.toBase58()}`);
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : String(err));
  process.exit(1);
});
