'use client';

import { useEffect, useMemo, useState } from 'react';

import { AppShell } from '@/components/AppShell';
import { ProofRail } from '@/components/ProofRail';
import { StatusBadge } from '@/components/StatusBadge';
import { getSettlements, getVerify, verifyHref } from '@/lib/api';
import { explorerTxUrl, formatLamports, formatWhen, shortHash } from '@/lib/format';
import type { SettlementIndex, VerifyPayload } from '@/lib/types';

export default function ExplorerPage() {
  const [rows, setRows] = useState<SettlementIndex[]>([]);
  const [source, setSource] = useState<'api' | 'fixture'>('api');
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [proof, setProof] = useState<VerifyPayload | null>(null);

  useEffect(() => {
    void getSettlements().then((result) => {
      setRows(result.data);
      setSource(result.source);
      const firstWithProof = result.data.find((row) => row.proof_hash);
      setSelectedId(firstWithProof?.id ?? result.data[0]?.id ?? null);
    });
  }, []);

  const selected = useMemo(
    () => rows.find((row) => row.id === selectedId) ?? null,
    [rows, selectedId],
  );

  useEffect(() => {
    let active = true;
    setProof(null);
    if (!selected?.proof_hash) {
      return () => {
        active = false;
      };
    }
    void getVerify(selected.proof_hash).then((result) => {
      if (active) setProof(result);
    });
    return () => {
      active = false;
    };
  }, [selected?.proof_hash]);

  return (
    <AppShell rail={<ProofRail proof={proof} fallbackHash={selected?.proof_hash} />}>
      <div className="page-head">
        <div>
          <h1>Settlement explorer</h1>
          <p className="lede">
            Transaction signatures, proof hashes, and the off-chain verify link auditors use to
            confirm the trigger rule ran as agreed.
          </p>
        </div>
        <p className="source-note">{source === 'api' ? 'API index' : 'Demo fixtures'}</p>
      </div>

      <div className="panel table-wrap">
        {rows.length === 0 ? (
          <p className="empty">No settlements indexed yet. Run `make db-seed` for local demo rows.</p>
        ) : (
          <table>
            <thead>
              <tr>
                <th>Select</th>
                <th>Status</th>
                <th>Tx</th>
                <th>Proof hash</th>
                <th>Payout</th>
                <th>Settled</th>
                <th>Verify</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => {
                const selectedRow = row.id === selectedId;
                return (
                  <tr
                    key={row.id}
                    style={{
                      background: selectedRow ? '#eef4f2' : undefined,
                    }}
                  >
                    <td>
                      <button
                        type="button"
                        className="select-btn"
                        aria-pressed={selectedRow}
                        onClick={() => setSelectedId(row.id)}
                      >
                        {selectedRow ? 'Selected' : 'Select'}
                      </button>
                    </td>
                    <td>
                      <StatusBadge status={row.status} />
                    </td>
                    <td className="hash">
                      {row.tx_signature ? (
                        <a href={explorerTxUrl(row.tx_signature)} target="_blank" rel="noreferrer">
                          {shortHash(row.tx_signature, 5)}
                        </a>
                      ) : (
                        '—'
                      )}
                    </td>
                    <td className="hash">{row.proof_hash ? shortHash(row.proof_hash) : '—'}</td>
                    <td>{formatLamports(row.payout_amount)}</td>
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

      {proof ? (
        <div className="panel proof-detail" style={{ marginTop: '1rem' }}>
          <div>
            <h3>Asset class</h3>
            <p>{proof.asset_class}</p>
          </div>
          <div>
            <h3>Risk score</h3>
            <p>
              {proof.risk_score} · {proof.scale}
            </p>
          </div>
          <div>
            <h3>Model confidence</h3>
            <p>{proof.model_confidence}</p>
          </div>
          <div>
            <h3>Timestamp</h3>
            <p>{formatWhen(proof.timestamp)}</p>
          </div>
        </div>
      ) : null}
    </AppShell>
  );
}
