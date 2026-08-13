import { describe, expect, it } from 'vitest';

import { evaluateTrigger } from '../src/evaluateTrigger.js';
import type { PriceFeed, TriggerRule } from '../src/types.js';

const NOW = 1_700_000_000;

function makeFeed(overrides: Partial<PriceFeed> = {}): PriceFeed {
  return {
    feedId: '0xtest',
    price: 90,
    conf: 1,
    expo: -8,
    publishTime: NOW - 10,
    ...overrides,
  };
}

const rule: TriggerRule = { threshold: 100, operator: 'lt' };

describe('evaluateTrigger', () => {
  it('triggers when price is below threshold and feed is valid', () => {
    const result = evaluateTrigger(makeFeed(), rule, NOW);

    expect(result.triggered).toBe(true);
    expect(result.reason).toBe('EVALUATED');
    expect(result.riskScore).toBeGreaterThan(0);
  });

  it('does not trigger when oracle is stale', () => {
    const result = evaluateTrigger(
      makeFeed({ publishTime: NOW - 120 }),
      rule,
      NOW,
    );

    expect(result.triggered).toBe(false);
    expect(result.reason).toBe('STALE_ORACLE');
  });

  it('does not trigger when confidence is too low', () => {
    const result = evaluateTrigger(
      makeFeed({ price: 100, conf: 20 }),
      rule,
      NOW,
    );

    expect(result.triggered).toBe(false);
    expect(result.reason).toBe('LOW_CONFIDENCE');
  });
});
