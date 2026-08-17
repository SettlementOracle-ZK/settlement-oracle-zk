'use client';

import { useEffect, useMemo, useState } from 'react';

import { AppShell } from '@/components/AppShell';
import { ProofRail } from '@/components/ProofRail';
import { getOracleLatest, getVerify } from '@/lib/api';
import { DEMO_PROOF_HASH } from '@/lib/fixtures';
import { formatUsd } from '@/lib/format';
import type { OracleFeed, VerifyPayload } from '@/lib/types';

function compare(price: number, threshold: number, operator: 'lt' | 'lte' | 'gt' | 'gte') {
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

export default function MonitorPage() {
  const [feed, setFeed] = useState<OracleFeed | null>(null);
  const [source, setSource] = useState<'api' | 'fixture'>('fixture');
  const [threshold, setThreshold] = useState(150);
  const [operator, setOperator] = useState<'lt' | 'lte' | 'gt' | 'gte'>('lt');
  const [proof, setProof] = useState<VerifyPayload | null>(null);

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
    const timer = window.setInterval(load, 8_000);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, []);

  useEffect(() => {
    let active = true;
    void getVerify(DEMO_PROOF_HASH).then((result) => {
      if (active) setProof(result);
    });
    return () => {
      active = false;
    };
  }, []);

  const wouldTrigger = feed ? compare(feed.price, threshold, operator) : false;
  const min = feed ? Math.min(feed.price, threshold) * 0.7 : 0;
  const max = feed ? Math.max(feed.price, threshold) * 1.15 : 1;
  const pricePct = feed ? ((feed.price - min) / (max - min)) * 100 : 0;
  const thresholdPct = ((threshold - min) / (max - min)) * 100;

  const warning = useMemo(() => {
    if (!feed) return null;
    if (feed.stale) {
      return {
        tone: 'critical' as const,
        text: `Oracle stale: last publish ${feed.age_seconds}s ago (max ${feed.max_staleness_seconds}s). Fail closed — no payout.`,
      };
    }
    if (feed.low_confidence) {
      return {
        tone: 'critical' as const,
        text: `Low confidence: conf/price exceeds ${feed.max_confidence_ratio}. Fail closed — no payout.`,
      };
    }
    if (feed.age_seconds > feed.max_staleness_seconds * 0.7) {
      return {
        tone: 'warn' as const,
        text: `Feed aging: ${feed.age_seconds}s old. Approaching the ${feed.max_staleness_seconds}s staleness gate.`,
      };
    }
    return null;
  }, [feed]);

  return (
    <AppShell rail={<ProofRail proof={proof} fallbackHash={DEMO_PROOF_HASH} />}>
      <div className="page-head">
        <div>
          <h1>Trigger monitor</h1>
          <p className="lede">
            Live SOL/USD against the policy threshold. Stale or low-confidence prints warn in
            amber; the settlement engine stays fail-closed.
          </p>
        </div>
        <p className="source-note">{source === 'api' ? 'Pyth via API' : 'Demo fixture'}</p>
      </div>

      <div className="panel">
        <div className="controls">
          <label>
            Threshold (USD)
            <input
              type="number"
              value={threshold}
              onChange={(event) => setThreshold(Number(event.target.value))}
            />
          </label>
          <label>
            Operator
            <select
              value={operator}
              onChange={(event) => setOperator(event.target.value as typeof operator)}
            >
              <option value="lt">price &lt; threshold</option>
              <option value="lte">price ≤ threshold</option>
              <option value="gt">price &gt; threshold</option>
              <option value="gte">price ≥ threshold</option>
            </select>
          </label>
        </div>

        {feed ? (
          <div className="barograph">
            <div className="barograph-scale" aria-hidden="true">
              <div className="barograph-fill" style={{ height: `${Math.max(pricePct, 4)}%` }} />
              <div className="barograph-threshold" style={{ bottom: `${thresholdPct}%` }} />
            </div>
            <div className="barograph-copy">
              <div>
                <p className="source-note">{feed.symbol}</p>
                <p className="metric">{formatUsd(feed.price)}</p>
                <p className="lede">
                  Confidence ±{formatUsd(feed.conf)} · age {feed.age_seconds}s
                </p>
              </div>
              <div>
                <p>
                  Strike {formatUsd(threshold)} · {operator.toUpperCase()} ·{' '}
                  {wouldTrigger ? 'would trigger' : 'inside band'}
                </p>
                <p className="source-note">This view does not submit on-chain payouts.</p>
              </div>
            </div>
          </div>
        ) : (
          <p className="empty">Waiting for oracle tick…</p>
        )}
      </div>

      {warning ? (
        <p className={warning.tone === 'critical' ? 'warn warn-critical' : 'warn'}>{warning.text}</p>
      ) : null}
    </AppShell>
  );
}
