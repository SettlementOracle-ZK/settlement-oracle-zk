import { describe, expect, it } from 'vitest';

import { MAX_CONFIDENCE_RATIO } from '../src/constants.js';
import { isLowConfidence } from '../src/validation.js';

describe('confidence validation', () => {
  it('rejects feed when confidence ratio exceeds threshold', () => {
    const price = 100;
    const conf = price * (MAX_CONFIDENCE_RATIO + 0.01);

    expect(isLowConfidence(price, conf)).toBe(true);
  });

  it('accepts feed when confidence ratio is within threshold', () => {
    const price = 100;
    const conf = price * (MAX_CONFIDENCE_RATIO - 0.01);

    expect(isLowConfidence(price, conf)).toBe(false);
  });
});
