import { describe, expect, it } from 'vitest';

import { MAX_STALENESS_SECONDS } from '../src/constants.js';
import { isStale } from '../src/validation.js';

describe('staleness validation', () => {
  it('rejects feed older than MAX_STALENESS_SECONDS', () => {
    const now = 1_700_000_000;
    const publishTime = now - MAX_STALENESS_SECONDS - 1;

    expect(isStale(publishTime, now)).toBe(true);
  });

  it('accepts feed within MAX_STALENESS_SECONDS window', () => {
    const now = 1_700_000_000;
    const publishTime = now - MAX_STALENESS_SECONDS + 5;

    expect(isStale(publishTime, now)).toBe(false);
  });
});
