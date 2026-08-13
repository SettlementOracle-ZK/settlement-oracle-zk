import { MAX_CONFIDENCE_RATIO, MAX_STALENESS_SECONDS } from './constants.js';

export function isStale(
  publishTime: number,
  nowSeconds: number,
  maxStalenessSeconds = MAX_STALENESS_SECONDS,
): boolean {
  return nowSeconds - publishTime > maxStalenessSeconds;
}

export function isLowConfidence(
  price: number,
  conf: number,
  maxRatio = MAX_CONFIDENCE_RATIO,
): boolean {
  if (price === 0) {
    return true;
  }
  return Math.abs(conf / price) > maxRatio;
}

export function normalizePythPrice(rawPrice: string, expo: number): number {
  return Number(rawPrice) * 10 ** expo;
}

export function computeRiskScore(price: number, threshold: number): number {
  if (threshold === 0) {
    return 0;
  }
  const ratio = Math.abs(price / threshold);
  return Math.min(100, Math.round(ratio * 100));
}
