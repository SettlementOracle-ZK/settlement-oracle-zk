import type { TriggerOperator } from './types.js';

const MAX_STALENESS_SECONDS = 60;
const MAX_CONFIDENCE_RATIO = 0.05;

function comparePrice(price: number, threshold: number, operator: TriggerOperator): boolean {
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

export function computeRiskScore(price: number, threshold: number): number {
  if (threshold === 0) return 0;
  return Math.min(100, Math.round((Math.abs(price / threshold)) * 100));
}

export function computeModelConfidence(price: number, conf: number): string {
  if (price === 0) return '0%';
  const ratio = Math.abs(conf / price);
  const pct = Math.max(0, Math.min(100, Math.round((1 - ratio) * 100)));
  return `${pct}%`;
}

export interface EvaluateInputs {
  feedId: string;
  oraclePrice: number;
  oracleConf: number;
  publishTime: number;
  threshold: number;
  operator: TriggerOperator;
  assetClass: string;
  nowSeconds?: number;
}

export function evaluateCircuit(inputs: EvaluateInputs): {
  triggered: boolean;
  risk_score: number;
  rejectReason?: 'INVALID_PRICE' | 'STALE_ORACLE' | 'LOW_CONFIDENCE';
} {
  const now = inputs.nowSeconds ?? Math.floor(Date.now() / 1000);

  if (inputs.oraclePrice === 0) {
    return { triggered: false, risk_score: 0, rejectReason: 'INVALID_PRICE' };
  }
  if (now - inputs.publishTime > MAX_STALENESS_SECONDS) {
    return { triggered: false, risk_score: 0, rejectReason: 'STALE_ORACLE' };
  }
  if (Math.abs(inputs.oracleConf / inputs.oraclePrice) > MAX_CONFIDENCE_RATIO) {
    return { triggered: false, risk_score: 0, rejectReason: 'LOW_CONFIDENCE' };
  }

  const triggered = comparePrice(inputs.oraclePrice, inputs.threshold, inputs.operator);
  const risk_score = computeRiskScore(inputs.oraclePrice, inputs.threshold);
  return { triggered, risk_score };
}

export function buildWitness(inputs: EvaluateInputs): import('./types.js').CircuitWitness {
  const result = evaluateCircuit(inputs);
  return {
    feed_id: inputs.feedId,
    oracle_price: inputs.oraclePrice,
    oracle_conf: inputs.oracleConf,
    publish_time: inputs.publishTime,
    threshold: inputs.threshold,
    operator: inputs.operator,
    triggered: result.triggered,
    risk_score: result.risk_score,
    asset_class: inputs.assetClass,
  };
}
