'use client';

import { useEffect, useMemo, useState } from 'react';

import { OracleSparkline, type SparkSample } from '@/components/OracleSparkline';
import { StrikeBoard } from '@/components/StrikeBoard';
import { compareTrigger, failClosed, oracleGateWarning } from '@/lib/trigger';
import { useOracleFeed } from '@/lib/useOracleFeed';

const MAX_SAMPLES = 40;

function chipLabel(args: {
  paused: boolean;
  status: string;
  meetsRule: boolean;
  gated: boolean;
}): string {
  const status = args.status.trim().toUpperCase();
  if (args.paused) return 'Paused — blocked';
  if (status === 'PAID') return 'Already paid';
  if (status === 'TRIGGERED') return 'Already triggered';
  if (status === 'FAILED') return 'Failed';
  if (args.meetsRule && args.gated) return 'Blocked (fail closed)';
  if (args.meetsRule) return 'Would trigger';
  return 'Inside band';
}

export function PolicyTriggerPanel({
  threshold,
  paused,
  status,
}: {
  threshold: number;
  paused: boolean;
  status: string;
}) {
  const { feed, source } = useOracleFeed();
  const [samples, setSamples] = useState<SparkSample[]>([]);

  useEffect(() => {
    if (!feed) return;
    setSamples((prev) => {
      const next = [...prev, { at: Date.now(), value: feed.price }];
      return next.length > MAX_SAMPLES ? next.slice(-MAX_SAMPLES) : next;
    });
  }, [feed]);

  const meetsRule = feed ? compareTrigger(feed.price, threshold) : false;
  const gated = failClosed(feed);
  const hot = meetsRule && !paused && !gated;
  const warning = useMemo(() => oracleGateWarning(feed), [feed]);
  const displaySamples =
    samples.length > 0 ? samples : feed ? [{ at: Date.now(), value: feed.price }] : [];

  return (
    <div className="panel">
      <div className="panel-kicker">
        <p className="source-note">Live vs this policy · delay ≥ {threshold} min</p>
        <p className="source-note">{source === 'api' ? 'Pyth via API' : 'Demo fixture'}</p>
      </div>
      {feed ? (
        <>
          <StrikeBoard
            feed={feed}
            threshold={threshold}
            hot={hot}
            hotLabel={chipLabel({ paused, status, meetsRule, gated })}
            legendLeft={`Index ${feed.price.toFixed(0)} min`}
            legendRight={`Trigger ${threshold} min`}
          />
          <OracleSparkline samples={displaySamples} threshold={threshold} />
        </>
      ) : (
        <p className="empty">Waiting for oracle tick…</p>
      )}
      {paused ? (
        <p className="warn warn-critical">Escrow is paused. Trigger evaluation will not run.</p>
      ) : null}
      {warning ? (
        <p className={warning.tone === 'critical' ? 'warn warn-critical' : 'warn'}>{warning.text}</p>
      ) : null}
    </div>
  );
}
