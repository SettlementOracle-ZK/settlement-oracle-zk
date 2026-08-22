import Link from 'next/link';

import { AppShell } from '@/components/AppShell';
import { HashChip } from '@/components/HashChip';
import { PolicyTriggerPanel } from '@/components/PolicyTriggerPanel';
import { ProofRail } from '@/components/ProofRail';
import { StatusBadge } from '@/components/StatusBadge';
import { getPolicy, getSettlements, getVerify, verifyHref } from '@/lib/api';
import {
  explorerTxUrl,
  formatDelayMinutes,
  formatLamports,
  formatUnix,
  formatWhen,
  isLikelySolanaSignature,
  samePolicyId,
} from '@/lib/format';

export const dynamic = 'force-dynamic';

export default async function PolicyDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const [{ data: policy, source }, settlements] = await Promise.all([
    getPolicy(id),
    getSettlements(),
  ]);

  if (!policy) {
    return (
      <AppShell>
        <div className="page-head">
          <div>
            <p className="kicker">On-chain escrow</p>
            <h1>Policy not found</h1>
            <p className="lede">
              No on-chain policy account for this id. It may not be indexed, or the escrow PDA was
              never initialized.
            </p>
          </div>
          <Link className="btn-primary" href="/policies">
            Back to policies
          </Link>
        </div>
      </AppShell>
    );
  }

  const related = settlements.data.filter((row) => samePolicyId(row.policy_id, policy.policy_id));
  const proofHash = related.find((row) => row.proof_hash)?.proof_hash ?? null;
  const proof = proofHash ? await getVerify(proofHash) : null;

  return (
    <AppShell rail={<ProofRail proof={proof} fallbackHash={proofHash} />}>
      <div className="page-head">
        <div>
          <p className="kicker">Flight delay · policy</p>
          <h1>Cover detail</h1>
          <p className="lede">
            On-chain escrow is source of truth for premium and trigger. Live delay is compared to
            this policy&apos;s threshold — stale or low-confidence feeds stay fail-closed.
          </p>
        </div>
        <div className="page-head-actions">
          <Link className="btn-primary" href="/policies">
            All policies
          </Link>
          <p className="source-note">{source === 'api' ? 'On-chain via API' : 'Demo fixtures'}</p>
        </div>
      </div>

      <div className="panel proof-detail">
        <div>
          <h3>Status</h3>
          <p className="status-row">
            <StatusBadge status={policy.escrow.status} />
            {policy.escrow.paused ? <span className="chip chip-halt">Paused</span> : null}
          </p>
        </div>
        <div>
          <h3>Escrow</h3>
          <p>{formatLamports(policy.escrow.amount)}</p>
        </div>
        <div>
          <h3>Trigger</h3>
          <p>delay &gt; {formatDelayMinutes(policy.escrow.trigger_threshold)}</p>
        </div>
        <div>
          <h3>Expiry</h3>
          <p>{formatUnix(policy.expiry)}</p>
        </div>
      </div>

      <PolicyTriggerPanel
        threshold={policy.escrow.trigger_threshold}
        paused={policy.escrow.paused}
        status={policy.escrow.status}
      />

      <div className="panel">
        <div className="panel-kicker">
          <p className="source-note">On-chain accounts</p>
        </div>
        <dl className="result-dl policy-dl">
          <dt>Policy id</dt>
          <dd>
            <HashChip value={`0x${policy.policy_id.replace(/^0x/i, '')}`} />
          </dd>
          <dt>Holder</dt>
          <dd>
            <HashChip value={policy.holder} size={5} />
          </dd>
          <dt>Authority</dt>
          <dd>
            <HashChip value={policy.escrow.authority} size={5} />
          </dd>
          <dt>Asset class</dt>
          <dd>
            <span className="chip">{policy.asset_class}</span>
          </dd>
          <dt>Policy PDA</dt>
          <dd>
            <HashChip value={policy.pdas.policy} size={5} />
          </dd>
          <dt>Escrow PDA</dt>
          <dd>
            <HashChip value={policy.pdas.escrow} size={5} />
          </dd>
        </dl>
      </div>

      <div className="panel table-wrap">
        <div className="panel-kicker">
          <p className="source-note">Related settlements</p>
        </div>
        {related.length === 0 ? (
          <p className="empty">No settlement indexed for this policy yet.</p>
        ) : (
          <table>
            <thead>
              <tr>
                <th>Status</th>
                <th>Payout</th>
                <th>Tx</th>
                <th>Proof</th>
                <th>Settled</th>
                <th>Verify</th>
              </tr>
            </thead>
            <tbody>
              {related.map((row) => {
                const txLink =
                  row.tx_signature && isLikelySolanaSignature(row.tx_signature)
                    ? explorerTxUrl(row.tx_signature)
                    : null;
                return (
                  <tr key={row.id}>
                    <td>
                      <StatusBadge status={row.status} />
                    </td>
                    <td className="mono">{formatLamports(row.payout_amount)}</td>
                    <td>
                      {row.tx_signature ? (
                        txLink ? (
                          <a className="link-btn" href={txLink} target="_blank" rel="noreferrer">
                            Explorer
                          </a>
                        ) : (
                          <HashChip value={row.tx_signature} size={4} />
                        )
                      ) : (
                        '—'
                      )}
                    </td>
                    <td>{row.proof_hash ? <HashChip value={row.proof_hash} /> : '—'}</td>
                    <td>{formatWhen(row.settled_at)}</td>
                    <td>
                      {row.proof_hash ? (
                        <a
                          className="link-btn"
                          href={row.verification_url ?? verifyHref(row.proof_hash)}
                          target="_blank"
                          rel="noreferrer"
                        >
                          Verify
                        </a>
                      ) : (
                        '—'
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}
      </div>
    </AppShell>
  );
}
