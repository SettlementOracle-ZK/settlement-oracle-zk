'use client';

import { useEffect, useState } from 'react';

import { getOracleLatest } from './api';
import type { OracleFeed } from './types';

const DEFAULT_INTERVAL_MS = 8_000;

export function useOracleFeed(intervalMs = DEFAULT_INTERVAL_MS) {
  const [feed, setFeed] = useState<OracleFeed | null>(null);
  const [source, setSource] = useState<'api' | 'fixture'>('api');

  useEffect(() => {
    let cancelled = false;
    const load = () => {
      void getOracleLatest().then((result) => {
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
  }, [intervalMs]);

  return { feed, source };
}
