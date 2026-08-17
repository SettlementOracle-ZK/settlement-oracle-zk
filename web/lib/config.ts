export const API_BASE =
  process.env.NEXT_PUBLIC_API_URL?.replace(/\/$/, '') ?? 'http://127.0.0.1:3000';

/** Explicit demo mode. Off by default so API failures never look like real proofs. */
export const USE_FIXTURES = process.env.NEXT_PUBLIC_USE_FIXTURES === 'true';
