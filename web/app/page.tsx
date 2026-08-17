import Link from 'next/link';

import { AppShell } from '@/components/AppShell';
import { ProofRail } from '@/components/ProofRail';
import { getOracleLatest, getPolicies, getSettlements, getVerify } from '@/lib/api';
import { DEMO_PROOF_HASH } from '@/lib/fixtures';
import { formatUsd } from '@/lib/format';

export const dynamic = 'force-dynamic';

export default async function HomePage() {
  const [policies, settlements, oracle, proof] = await Promise.all([
    getPolicies(),
    getSettlements(),
    getOracleLatest(),
    getVerify(DEMO_PROOF_HASH),
  ]);

  const feed = oracle.data;
  const paid = settlements.data.filter((row) => row.status.toUpperCase() === 'PAID').length;

  return (
    <AppShell rail={<ProofRail proof={proof} fallbackHash={DEMO_PROOF_HASH} />}>
      <div className="hero">
        <p className="kicker">Parametric settlement · Solana</p>
        <h1>Vaults. Triggers. Proofs.</h1>
        <p className="lede">
          Premiums stay in on-chain escrow. Pyth prints the strike. A ZK hash attests the rule ran
          as agreed — no off-chain payouts.
        </p>
      </div>

      <div className="stat-grid">
        <div className="stat">
          <span className="stat-label">Policies indexed</span>
          <span className="stat-value">{policies.data.length}</span>
        </div>
        <div className="stat">
          <span className="stat-label">Settlements · paid</span>
          <span className="stat-value">
            {settlements.data.length}
            <span className="stat-sub"> · {paid} paid</span>
          </span>
        </div>
        <div className="stat">
          <span className="stat-label">{feed?.symbol ?? 'Oracle'}</span>
          <span className={`stat-value ${feed?.stale ? 'stat-value-halt' : 'stat-value-live'}`}>
            {feed ? formatUsd(feed.price) : '—'}
          </span>
        </div>
      </div>

      <div className="desk-grid">
        <Link className="desk-card" href="/policies">
          <small>01 · Escrow</small>
          <h2>Policies</h2>
          <p>Active covers, holders, and vault PDAs. On-chain balances stay the source of truth.</p>
        </Link>
        <Link className="desk-card" href="/monitor">
          <small>02 · Oracle</small>
          <h2>Monitor</h2>
          <p>Live SOL/USD vs strike. Stale or low-confidence feeds fail closed — no payout.</p>
        </Link>
        <Link className="desk-card" href="/explorer">
          <small>03 · Attestation</small>
          <h2>Explorer</h2>
          <p>Tx signatures, proof hashes, and the /verify link auditors use to check the trigger.</p>
        </Link>
      </div>
    </AppShell>
  );
}
