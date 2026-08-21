import type { DeskStatus } from './types';

export function canonicalPolicyId(id: string): string {
  return id.trim().replace(/^0x/i, '').toLowerCase();
}

export function samePolicyId(a: string, b: string): boolean {
  return canonicalPolicyId(a) === canonicalPolicyId(b);
}

export function normalizeStatus(raw: string): DeskStatus {
  const value = raw.trim().toUpperCase();
  if (value === 'TRIGGERED') return 'TRIGGERED';
  if (value === 'PAID') return 'PAID';
  if (value === 'FAILED') return 'FAILED';
  if (value === 'ACTIVE') return 'ACTIVE';
  return 'PENDING';
}

export function formatUnix(seconds: number | null | undefined): string {
  if (seconds == null || !Number.isFinite(seconds)) return '—';
  return formatWhen(new Date(seconds * 1000).toISOString());
}

export function formatDelayMinutes(minutes: number | null | undefined): string {
  if (minutes == null || !Number.isFinite(minutes)) return '—';
  if (minutes >= 60 && minutes % 60 === 0) {
    return `${minutes} min (${minutes / 60}h)`;
  }
  return `${minutes} min`;
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
