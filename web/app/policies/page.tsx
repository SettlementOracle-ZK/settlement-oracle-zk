import { AppShell } from '@/components/AppShell';
import { ProofRail } from '@/components/ProofRail';
import { getPolicies, getVerify } from '@/lib/api';
import { DEMO_PROOF_HASH } from '@/lib/fixtures';
import { formatWhen, shortHash } from '@/lib/format';

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
          <h1>Active policies</h1>
          <p className="lede">
            Parametric covers on the blotter. Escrow balances stay on-chain; this desk indexes
            status for actuaries and auditors.
          </p>
        </div>
        <p className="source-note">{source === 'api' ? 'API index' : 'Demo fixtures'}</p>
      </div>
      <div className="panel table-wrap">
        {data.length === 0 ? (
          <p className="empty">No policies indexed. Run `make db-seed` or create a policy on-chain.</p>
        ) : (
          <table>
            <thead>
              <tr>
                <th>Holder</th>
                <th>Class</th>
                <th>Policy id</th>
                <th>Expiry</th>
                <th>Escrow PDA</th>
              </tr>
            </thead>
            <tbody>
              {data.map((policy) => (
                <tr key={policy.policy_id}>
                  <td>{policy.holder}</td>
                  <td>{policy.asset_class}</td>
                  <td className="hash">{shortHash(`0x${policy.policy_id}`)}</td>
                  <td>{formatWhen(policy.expiry)}</td>
                  <td className="hash">{shortHash(policy.escrow_pda, 4)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </AppShell>
  );
}
