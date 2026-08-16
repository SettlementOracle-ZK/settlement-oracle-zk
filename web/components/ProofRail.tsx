import { shortHash } from '@/lib/format';
import type { VerifyPayload } from '@/lib/types';

export function ProofRail({
  proof,
  fallbackHash,
}: {
  proof: VerifyPayload | null;
  fallbackHash?: string | null;
}) {
  const hash = proof?.zk_proof.hash ?? fallbackHash ?? '— awaiting attestation —';
  const chunks = hash.replace(/^0x/, '').match(/.{1,8}/g) ?? [hash];

  return (
    <aside className="rail" aria-label="Attestation rail">
      <h2>Attestation</h2>
      <div className="seal" aria-hidden="true">
        {chunks.map((chunk) => (
          <div key={chunk}>{chunk}</div>
        ))}
      </div>
      <dl>
        <dt>Proof hash</dt>
        <dd className="mono">{shortHash(hash, 8)}</dd>
        <dt>Status</dt>
        <dd>{proof?.verified ? 'Verified (off-chain)' : 'Not loaded'}</dd>
        {proof ? (
          <>
            <dt>Risk score</dt>
            <dd>
              {proof.risk_score} / {proof.scale}
            </dd>
            <dt>Verify</dt>
            <dd>
              <a href={proof.zk_proof.verification_url} target="_blank" rel="noreferrer">
                Open /verify
              </a>
            </dd>
          </>
        ) : null}
      </dl>
    </aside>
  );
}
