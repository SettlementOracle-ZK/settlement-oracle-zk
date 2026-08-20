import type { SettlementStatus } from './types';

export function normalizeStatus(raw: string): SettlementStatus {
  const value = raw.trim().toUpperCase();
  if (value === 'TRIGGERED') return 'TRIGGERED';
  if (value === 'PAID') return 'PAID';
  if (value === 'FAILED') return 'FAILED';
  return 'PENDING';
}

export function shortHash(value: string, size = 6): string {
  const body = value.startsWith('0x') ? value.slice(2) : value;
  if (body.length <= size * 2) return value;
  return `${value.slice(0, value.startsWith('0x') ? size + 2 : size)}…${body.slice(-size)}`;
}

export function isLikelySolanaSignature(value: string): boolean {
  const sig = value.trim();
  // Real Solana tx signatures are base58 and typically 87–88 chars.
  return sig.length >= 80 && sig.length <= 128 && /^[1-9A-HJ-NP-Za-km-z]+$/.test(sig);
}

export function explorerTxUrl(signature: string): string {
  return `https://explorer.solana.com/tx/${signature}?cluster=devnet`;
}

export function formatLamports(lamports: number | null | undefined): string {
  if (lamports == null) return '—';
  return `${(lamports / 1_000_000_000).toFixed(4)} SOL`;
}

export function formatUsd(price: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    maximumFractionDigits: 2,
  }).format(price);
}

export function formatWhen(iso: string | null | undefined): string {
  if (!iso) return '—';
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleString('en-GB', { timeZone: 'UTC', hour12: false }) + ' UTC';
}
