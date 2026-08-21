'use client';

import { useEffect, useMemo, useState } from 'react';

import { AppShell } from '@/components/AppShell';
import { ProofRail } from '@/components/ProofRail';
import { StrikeBoard } from '@/components/StrikeBoard';
import { getVerify } from '@/lib/api';
import { DEMO_PROOF_HASH } from '@/lib/fixtures';
import { compareTrigger, oracleGateWarning, type TriggerOperator } from '@/lib/trigger';
import { useOracleFeed } from '@/lib/useOracleFeed';
import type { VerifyPayload } from '@/lib/types';

export default function MonitorPage() {
  const { feed, source } = useOracleFeed();
  const [threshold, setThreshold] = useState(120);
  const [operator, setOperator] = useState<TriggerOperator>('gte');
  const [proof, setProof] = useState<VerifyPayload | null>(null);

  useEffect(() => {
    let active = true;
    void getVerify(DEMO_PROOF_HASH).then((result) => {
      if (active) setProof(result);
    });
    return () => {
      active = false;
    };
  }, []);

  const wouldTrigger = feed ? compareTrigger(feed.price, threshold, operator) : false;
  const warning = useMemo(() => oracleGateWarning(feed), [feed]);

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
              onChange={(event) => setOperator(event.target.value as TriggerOperator)}
            >
              <option value="lt">price &lt; strike</option>
              <option value="lte">price ≤ strike</option>
              <option value="gt">price &gt; strike</option>
              <option value="gte">price ≥ strike</option>
            </select>
          </label>
        </div>

        {feed ? (
          <StrikeBoard
            feed={feed}
            threshold={threshold}
            hot={wouldTrigger}
            hotLabel={wouldTrigger ? 'Would trigger' : 'Inside band'}
            legendLeft={`Index ${feed.price.toFixed(0)} min`}
            legendRight={`Trigger ${threshold} min`}
          />
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
