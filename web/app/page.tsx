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
        <p className="kicker">Parametric travel · Solana</p>
        <h1>Flights. Delays. Proofs.</h1>
        <p className="lede">
          Register a flight, escrow the premium on-chain, and let the oracle attest delay triggers.
          A ZK hash proves the rule ran — automatic payout, no claim form.
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
        <Link className="desk-card" href="/policies/new">
          <small>01 · Register</small>
          <h2>New cover</h2>
          <p>Connect Phantom on devnet and create a real flight-delay policy — escrow + premium in one flow.</p>
        </Link>
        <Link className="desk-card" href="/policies">
          <small>02 · Escrow</small>
          <h2>Policies</h2>
          <p>Active flight covers, holders, and vault PDAs. On-chain balances stay the source of truth.</p>
        </Link>
        <Link className="desk-card" href="/monitor">
          <small>03 · Oracle</small>
          <h2>Monitor</h2>
          <p>Delay index vs trigger. Stale or low-confidence feeds fail closed — no payout.</p>
        </Link>
        <Link className="desk-card" href="/explorer">
          <small>04 · Attestation</small>
          <h2>Explorer</h2>
          <p>Tx signatures, proof hashes, and the /verify link auditors use to check the trigger.</p>
        </Link>
      </div>
    </AppShell>
  );
}
