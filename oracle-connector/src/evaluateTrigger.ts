import { MAX_CONFIDENCE_RATIO, MAX_STALENESS_SECONDS } from './constants.js';
import type { PriceFeed, TriggerResult, TriggerRule } from './types.js';
import { computeRiskScore, isLowConfidence, isStale } from './validation.js';

function comparePrice(price: number, threshold: number, operator: TriggerRule['operator']): boolean {
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

export function evaluateTrigger(
  feed: PriceFeed,
  rule: TriggerRule,
  nowSeconds = Math.floor(Date.now() / 1000),
): TriggerResult {
  const timestamp = new Date(nowSeconds * 1000).toISOString();

  if (feed.price === 0) {
    return { triggered: false, reason: 'INVALID_PRICE', riskScore: 0, timestamp };
  }

  if (isStale(feed.publishTime, nowSeconds, MAX_STALENESS_SECONDS)) {
    return { triggered: false, reason: 'STALE_ORACLE', riskScore: 0, timestamp };
  }

  if (isLowConfidence(feed.price, feed.conf, MAX_CONFIDENCE_RATIO)) {
    return { triggered: false, reason: 'LOW_CONFIDENCE', riskScore: 0, timestamp };
  }

  const triggered = comparePrice(feed.price, rule.threshold, rule.operator);
  const riskScore = computeRiskScore(feed.price, rule.threshold);

  return {
    triggered,
    reason: 'EVALUATED',
    riskScore,
    timestamp,
  };
}
