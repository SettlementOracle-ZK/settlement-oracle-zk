import BN from 'bn.js';
import { SystemProgram } from '@solana/web3.js';
import type { Connection } from '@solana/web3.js';
import type { AnchorWallet } from '@solana/wallet-adapter-react';

import {
  ASSET_CLASS_FLIGHT_DELAY,
  DEFAULT_PREMIUM_LAMPORTS,
  DEFAULT_TRIGGER_THRESHOLD,
  encodeAssetClass,
  policyExpiryOneYear,
} from '@/lib/domain';

import { escrowPda, policyIdHex, policyPda, randomPolicyId } from './pdas';
import { getEscrowProgram } from './program';

export type CreateFlightPolicyInput = {
  premiumLamports?: number;
  assetClass?: string;
  /** Delay trigger in minutes (e.g. 120 = 2 hours). */
  triggerThresholdMinutes?: number;
};

export type CreateFlightPolicyResult = {
  policyId: Uint8Array;
  policyIdHex: string;
  policyPda: string;
  escrowPda: string;
  signatures: {
    initializePolicy: string;
    initializeEscrow: string;
    depositPremium: string;
  };
};

export async function createFlightPolicyOnChain(
  connection: Connection,
  wallet: AnchorWallet,
  input: CreateFlightPolicyInput,
): Promise<CreateFlightPolicyResult> {
  const program = getEscrowProgram(connection, wallet);
  const policyId = randomPolicyId();
  const policy = policyPda(program.programId, policyId);
  const escrow = escrowPda(program.programId, policyId);
  const expiry = new BN(policyExpiryOneYear());
  const assetClass = encodeAssetClass(input.assetClass ?? ASSET_CLASS_FLIGHT_DELAY);
  const deposit = new BN(input.premiumLamports ?? DEFAULT_PREMIUM_LAMPORTS);
  const triggerThreshold = new BN(
    input.triggerThresholdMinutes ?? Number(DEFAULT_TRIGGER_THRESHOLD),
  );

  const sig1 = await program.methods
    .initializePolicy([...policyId], wallet.publicKey, expiry, assetClass)
    .accounts({
      authority: wallet.publicKey,
      policy,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  const sig2 = await program.methods
    .initializeEscrow([...policyId], triggerThreshold)
    .accounts({
      authority: wallet.publicKey,
      policy,
      escrow,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  const sig3 = await program.methods
    .depositPremium(deposit)
    .accounts({
      authority: wallet.publicKey,
      escrow,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  return {
    policyId,
    policyIdHex: policyIdHex(policyId),
    policyPda: policy.toBase58(),
    escrowPda: escrow.toBase58(),
    signatures: {
      initializePolicy: sig1,
      initializeEscrow: sig2,
      depositPremium: sig3,
    },
  };
}
