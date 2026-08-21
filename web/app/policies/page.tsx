import Link from 'next/link';

import { AppShell } from '@/components/AppShell';
import { HashChip } from '@/components/HashChip';
import { ProofRail } from '@/components/ProofRail';
import { getPolicies, getVerify } from '@/lib/api';
import { DEMO_PROOF_HASH } from '@/lib/fixtures';
import { formatWhen } from '@/lib/format';

export const dynamic = 'force-dynamic';

export default async function PoliciesPage() {
  const [{ data, source }, proof] = await Promise.all([
    getPolicies(),
    getVerify(DEMO_PROOF_HASH),
  ]);

  return (
    <AppShell rail={<ProofRail proof={proof} fallbackHash={DEMO_PROOF_HASH} />}>
      <div className="page-head">
        <div>
          <p className="kicker">On-chain escrow</p>
          <h1>Flight delay policies</h1>
          <p className="lede">
            Parametric travel covers indexed for the desk. Escrow PDAs hold premium; this view never
            moves funds.
          </p>
        </div>
        <div className="page-head-actions">
          <Link className="btn-primary" href="/policies/new">
            Register cover
          </Link>
          <p className="source-note">{source === 'api' ? 'API index' : 'Demo fixtures'}</p>
        </div>
      </div>
      <div className="panel table-wrap">
        {data.length === 0 ? (
          <p className="empty">
            No policies indexed.{' '}
            <Link href="/policies/new">Register a flight cover on devnet</Link> or run{' '}
            <code>make db-seed</code>.
          </p>
        ) : (
          <table>
            <thead>
              <tr>
                <th>Holder</th>
                <th>Class</th>
                <th>Policy id</th>
                <th>Policy PDA</th>
                <th>Expiry</th>
                <th>Escrow PDA</th>
              </tr>
            </thead>
            <tbody>
              {data.map((policy) => (
                <tr key={policy.policy_id}>
                  <td>
                    <HashChip value={policy.holder} size={4} />
                  </td>
                  <td>
                    <span className="chip">{policy.asset_class}</span>
                  </td>
                  <td>
                    <HashChip value={`0x${policy.policy_id}`} />
                  </td>
                  <td>
                    <HashChip value={policy.policy_pda} size={4} />
                  </td>
                  <td>{formatWhen(policy.expiry)}</td>
                  <td>
                    <HashChip value={policy.escrow_pda} size={4} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </AppShell>
  );
}
