'use client';

import { useEffect, useMemo, useState } from 'react';

import { AppShell } from '@/components/AppShell';
import { ProofRail } from '@/components/ProofRail';
import { getOracleLatest, getVerify } from '@/lib/api';
import { DEMO_PROOF_HASH } from '@/lib/fixtures';
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

function clampPct(value: number) {
  return Math.min(100, Math.max(0, value));
}

export default function MonitorPage() {
  const [feed, setFeed] = useState<OracleFeed | null>(null);
  const [source, setSource] = useState<'api' | 'fixture'>('fixture');
  const [threshold, setThreshold] = useState(120);
  const [operator, setOperator] = useState<'lt' | 'lte' | 'gt' | 'gte'>('gte');
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
  const span = max - min || 1;
  const pricePct = feed ? clampPct(((feed.price - min) / span) * 100) : 0;
  const thresholdPct = clampPct(((threshold - min) / span) * 100);

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
          <p className="kicker">Flight delay · oracle stand-in</p>
          <h1>Trigger monitor</h1>
          <p className="lede">
            Reported delay index versus your registered trigger (e.g. 2+ hours). MVP uses a Pyth
            tick as devnet stand-in — stale or low-confidence feeds stay fail-closed.
          </p>
        </div>
        <p className="source-note">{source === 'api' ? 'Pyth via API' : 'Demo fixture'}</p>
      </div>

      <div className="panel">
        <div className="controls">
          <label>
            Delay trigger (min)
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
              <option value="lt">price &lt; strike</option>
              <option value="lte">price ≤ strike</option>
              <option value="gt">price &gt; strike</option>
              <option value="gte">price ≥ strike</option>
            </select>
          </label>
        </div>

        {feed ? (
          <div className="strike-board">
            <div className="strike-meta">
              <div>
                <p className="source-note">{feed.symbol}</p>
                <p className={`metric ${feed.stale ? '' : 'metric-live'}`}>
                  {feed.price.toFixed(0)} min
                </p>
                <p className="lede">
                  Confidence ±{feed.conf.toFixed(2)} · age {feed.age_seconds}s
                </p>
              </div>
              <span className="fire-chip" data-hot={wouldTrigger}>
                {wouldTrigger ? 'Would trigger' : 'Inside band'}
              </span>
            </div>
            <div className="strike-track" aria-hidden="true">
              <div className="strike-fill" style={{ width: `${Math.max(pricePct, 3)}%` }} />
              <div className="strike-mark" style={{ left: `${thresholdPct}%` }} />
              <div className="strike-spot" style={{ left: `${pricePct}%` }} />
            </div>
            <div className="strike-legend">
              <span>Index {feed.price.toFixed(0)} min</span>
              <span>Trigger {threshold} min</span>
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
