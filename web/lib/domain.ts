/** Parametric flight delay cover (MVP demo — oracle stand-in on devnet). */
export const ASSET_CLASS_FLIGHT_DELAY = 'flight_delay';

/** On-chain trigger threshold in minutes (default 2h delay). */
export const DEFAULT_TRIGGER_THRESHOLD = '120';

/** Default mock oracle delay for devnet demos (150 min — triggers payout vs 120 min threshold). */
export const DEFAULT_MOCK_DELAY_MINUTES = 150;

/** Default premium deposit for devnet demos (0.00001 SOL). */
export const DEFAULT_PREMIUM_LAMPORTS = 10_000;

export function encodeAssetClass(label: string): number[] {
  const bytes = new Uint8Array(32);
  const encoded = new TextEncoder().encode(label.slice(0, 32));
  bytes.set(encoded);
  return [...bytes];
}

export function policyExpiryOneYear(): number {
  return Math.floor(Date.now() / 1000) + 365 * 24 * 60 * 60;
}

export function policyExpiryRfc3339(): string {
  return new Date(policyExpiryOneYear() * 1000).toISOString();
}
