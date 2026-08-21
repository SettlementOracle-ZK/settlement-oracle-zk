import type { OracleFeed } from './types';

export type TriggerOperator = 'lt' | 'lte' | 'gt' | 'gte';

/** On-chain `evaluate_trigger` for the MVP delay stand-in. */
export const ON_CHAIN_OPERATOR: TriggerOperator = 'gte';

export function compareTrigger(
  price: number,
  threshold: number,
  operator: TriggerOperator = ON_CHAIN_OPERATOR,
): boolean {
  switch (operator) {
    case 'lt':
      return price < threshold;
    case 'lte':
      return price <= threshold;
    case 'gt':
      return price > threshold;
    case 'gte':
      return price >= threshold;
    default:
      return false;
  }
}

export function clampPct(value: number): number {
  return Math.min(100, Math.max(0, value));
}

export function strikePercents(price: number, threshold: number): { pricePct: number; thresholdPct: number } {
  const min = Math.min(price, threshold) * 0.7;
  const max = Math.max(price, threshold) * 1.15;
  const span = max - min || 1;
  return {
    pricePct: clampPct(((price - min) / span) * 100),
    thresholdPct: clampPct(((threshold - min) / span) * 100),
  };
}

export function oracleGateWarning(feed: OracleFeed | null): {
  tone: 'critical' | 'warn';
  text: string;
} | null {
  if (!feed) return null;
  if (feed.stale) {
    return {
      tone: 'critical',
      text: `Oracle stale: last publish ${feed.age_seconds}s ago (max ${feed.max_staleness_seconds}s). Fail closed — no payout.`,
    };
  }
  if (feed.low_confidence) {
    return {
      tone: 'critical',
      text: `Low confidence: conf/price exceeds ${feed.max_confidence_ratio}. Fail closed — no payout.`,
    };
  }
  if (feed.age_seconds > feed.max_staleness_seconds * 0.7) {
    return {
      tone: 'warn',
      text: `Feed aging: ${feed.age_seconds}s old. Approaching the ${feed.max_staleness_seconds}s staleness gate.`,
    };
  }
  return null;
}

export function failClosed(feed: OracleFeed | null): boolean {
  return !feed || feed.stale || feed.low_confidence;
}
