import { AnchorProvider, Program, type Idl } from '@coral-xyz/anchor';
import type { Connection } from '@solana/web3.js';
import type { AnchorWallet } from '@solana/wallet-adapter-react';

import { ESCROW_PROGRAM_ID } from '@/lib/config';
import idl from '@/lib/idl/escrow.json';

type EscrowIdl = Idl & { address?: string };

export function getEscrowProgram(connection: Connection, wallet: AnchorWallet): Program {
  const provider = new AnchorProvider(connection, wallet, {
    commitment: 'confirmed',
    preflightCommitment: 'confirmed',
  });
  const programIdl = { ...(idl as EscrowIdl), address: ESCROW_PROGRAM_ID };
  return new Program(programIdl, provider);
}
