'use client';

import { useEffect, useState } from 'react';

import { getOracleLatest } from '@/lib/api';
import { formatUsd } from '@/lib/format';
import type { OracleFeed } from '@/lib/types';

export function OracleTicker() {
  const [feed, setFeed] = useState<OracleFeed | null>(null);

  useEffect(() => {
    let cancelled = false;
    const load = () => {
      void getOracleLatest().then((result) => {
        if (!cancelled) setFeed(result.data);
      });
    };
    load();
    const timer = window.setInterval(load, 8_000);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, []);

  if (!feed) {
    return (
      <span className="ticker">
        <span className="ticker-dot" />
        oracle · syncing
      </span>
    );
  }

  const tone = feed.stale || feed.low_confidence ? 'ticker-stale' : 'ticker-live';
  return (
    <span className={`ticker ${tone}`} title={`${feed.symbol} · age ${feed.age_seconds}s`}>
      <span className="ticker-dot" />
      {feed.symbol} {formatUsd(feed.price)}
    </span>
  );
}
