'use client';

import { useEffect, useState } from 'react';

import { getOracleDelay, getOracleLatest } from './api';
import type { OracleFeed } from './types';

const DEFAULT_INTERVAL_MS = 8_000;

type FeedKind = 'latest' | 'delay';

function useOraclePoll(kind: FeedKind, intervalMs = DEFAULT_INTERVAL_MS) {
  const [feed, setFeed] = useState<OracleFeed | null>(null);
  const [source, setSource] = useState<'api' | 'fixture'>('api');

  useEffect(() => {
    let cancelled = false;
    const load = () => {
      const promise = kind === 'delay' ? getOracleDelay() : getOracleLatest();
      void promise.then((result) => {
        if (cancelled) return;
        setFeed(result.data);
        setSource(result.source);
      });
    };
    load();
    const timer = window.setInterval(load, intervalMs);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [kind, intervalMs]);

  return { feed, source };
}

export function useOracleFeed(intervalMs = DEFAULT_INTERVAL_MS) {
  return useOraclePoll('latest', intervalMs);
}

/** Flight-delay stand-in: reads program mock Pyth PDA (delay in minutes). */
export function useDelayFeed(intervalMs = DEFAULT_INTERVAL_MS) {
  return useOraclePoll('delay', intervalMs);
}
