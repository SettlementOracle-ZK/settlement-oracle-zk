export const API_BASE =
  process.env.NEXT_PUBLIC_API_URL?.replace(/\/$/, '') ?? 'http://127.0.0.1:3000';

export const ESCROW_PROGRAM_ID =
  process.env.NEXT_PUBLIC_ESCROW_PROGRAM_ID ??
  'DqdgQv57RWZ4RVUQ7v6SMvTfA11gFph2TNKZDz7sa3Ap';

export const SOLANA_RPC =
  process.env.NEXT_PUBLIC_SOLANA_RPC_URL ?? 'https://api.devnet.solana.com';

/** Explicit demo mode. Off by default so API failures never look like real proofs. */
export const USE_FIXTURES = process.env.NEXT_PUBLIC_USE_FIXTURES === 'true';
