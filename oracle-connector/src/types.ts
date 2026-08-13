export type TriggerOperator = 'lt' | 'lte' | 'gt' | 'gte';

export interface PriceFeed {
  feedId: string;
  price: number;
  conf: number;
  expo: number;
  publishTime: number;
}

export interface TriggerRule {
  threshold: number;
  operator: TriggerOperator;
}

export type TriggerReason =
  | 'STALE_ORACLE'
  | 'LOW_CONFIDENCE'
  | 'EVALUATED'
  | 'INVALID_PRICE';

export interface TriggerResult {
  triggered: boolean;
  reason: TriggerReason;
  riskScore: number;
  timestamp: string;
}

export interface PythFeedSnapshot {
  id: string;
  price: {
    price: string;
    conf: string;
    expo: number;
    publish_time: number;
  };
}
