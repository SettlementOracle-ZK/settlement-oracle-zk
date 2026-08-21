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
            Indexed covers only. Open a row for trigger vs live delay, escrow, and PDAs — this list
            never moves funds.
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
            <Link href="/policies/new">Register a flight cover</Link> or run <code>make db-seed</code>.
          </p>
        ) : (
          <table className="policy-table">
            <thead>
              <tr>
                <th>Cover</th>
                <th>Identity</th>
                <th>Expiry</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {data.map((policy) => (
                <tr key={policy.policy_id}>
                  <td>
                    <span className="chip">{policy.asset_class.replaceAll('_', ' ')}</span>
                  </td>
                  <td>
                    <div className="id-stack">
                      <HashChip label="Holder" value={policy.holder} size={4} />
                      <HashChip label="Policy" value={`0x${policy.policy_id}`} size={4} />
                    </div>
                  </td>
                  <td className="policy-expiry">{formatWhen(policy.expiry)}</td>
                  <td className="policy-action">
                    <Link className="btn-open" href={`/policies/${policy.policy_id}`}>
                      Open
                    </Link>
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
