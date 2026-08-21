'use client';

import Link from 'next/link';
import { useCallback, useState } from 'react';
import { useAnchorWallet, useConnection } from '@solana/wallet-adapter-react';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';

import { registerPolicy } from '@/lib/api';
import {
  ASSET_CLASS_FLIGHT_DELAY,
  DEFAULT_PREMIUM_LAMPORTS,
  policyExpiryRfc3339,
} from '@/lib/domain';
import { createFlightPolicyOnChain } from '@/lib/escrow/createPolicy';

type Step = 'idle' | 'chain' | 'index' | 'done' | 'error';

export function CreatePolicyForm() {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();
  const [flightNumber, setFlightNumber] = useState('LA456');
  const [route, setRoute] = useState('GRU → MIA');
  const [delayHours, setDelayHours] = useState(2);
  const [premiumSol, setPremiumSol] = useState(DEFAULT_PREMIUM_LAMPORTS / LAMPORTS_PER_SOL);
  const [step, setStep] = useState<Step>('idle');
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<{
    policyIdHex: string;
    policyPda: string;
    escrowPda: string;
    payoutTx?: string;
  } | null>(null);

  const premiumLamports = Math.max(1, Math.round(premiumSol * LAMPORTS_PER_SOL));

  const onSubmit = useCallback(async () => {
    setError(null);
    if (!wallet?.publicKey) {
      setError('Connect Phantom or Solflare on devnet first.');
      return;
    }
    if (!flightNumber.trim()) {
      setError('Enter a flight number to register.');
      return;
    }

    try {
      setStep('chain');
      const chain = await createFlightPolicyOnChain(connection, wallet, {
        premiumLamports,
        assetClass: ASSET_CLASS_FLIGHT_DELAY,
        triggerThresholdMinutes: delayHours * 60,
      });

      setStep('index');
      await registerPolicy({
        policy_id: chain.policyIdHex,
        holder: wallet.publicKey.toBase58(),
        expiry: policyExpiryRfc3339(),
        asset_class: ASSET_CLASS_FLIGHT_DELAY,
        policy_pda: chain.policyPda,
        escrow_pda: chain.escrowPda,
        init_policy_tx: chain.signatures.initializePolicy,
      });

      setResult({
        policyIdHex: chain.policyIdHex,
        policyPda: chain.policyPda,
        escrowPda: chain.escrowPda,
        payoutTx: chain.signatures.depositPremium,
      });
      setStep('done');
    } catch (err) {
      setStep('error');
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [connection, wallet, flightNumber, premiumLamports]);

  if (step === 'done' && result) {
    return (
      <div className="panel form-panel">
        <h2 className="form-title">Cover registered</h2>
        <p className="lede">
          Flight <strong>{flightNumber}</strong> ({route}) — parametric delay cover is on devnet.
          Premium escrowed; payout fires when the oracle attests the delay trigger.
        </p>
        <dl className="result-dl">
          <dt>Policy id</dt>
          <dd className="mono">0x{result.policyIdHex}</dd>
          <dt>Policy PDA</dt>
          <dd className="mono">{result.policyPda}</dd>
          <dt>Delay trigger (demo)</dt>
          <dd>{delayHours}+ hours (indexed oracle rule)</dd>
          <dt>Escrow PDA</dt>
          <dd className="mono">{result.escrowPda}</dd>
          <dt>Deposit tx</dt>
          <dd>
            <a
              href={`https://explorer.solana.com/tx/${result.payoutTx}?cluster=devnet`}
              target="_blank"
              rel="noreferrer"
            >
              View on Explorer
            </a>
          </dd>
        </dl>
        <div className="form-actions">
          <Link className="btn-primary" href="/policies">
            Back to policies
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="panel form-panel">
      <div className="controls">
        <label>
          Flight number
          <input
            value={flightNumber}
            onChange={(e) => setFlightNumber(e.target.value.toUpperCase())}
            placeholder="LA456"
          />
        </label>
        <label>
          Route
          <input value={route} onChange={(e) => setRoute(e.target.value)} placeholder="GRU → MIA" />
        </label>
        <label>
          Delay trigger (hours)
          <input
            type="number"
            min={1}
            max={24}
            value={delayHours}
            onChange={(e) => setDelayHours(Number(e.target.value))}
          />
        </label>
        <label>
          Premium (SOL)
          <input
            type="number"
            min={0.000001}
            step={0.000001}
            value={premiumSol}
            onChange={(e) => setPremiumSol(Number(e.target.value))}
          />
        </label>
      </div>

      <p className="lede form-note">
        Beneficiary wallet: connected account (receives automatic payout). Three devnet
        transactions: policy → escrow → premium deposit (~0.005 SOL fees + premium).
      </p>

      {error ? <p className="warn warn-critical">{error}</p> : null}

      <div className="form-actions">
        <button
          type="button"
          className="btn-primary"
          disabled={!wallet?.publicKey || step === 'chain' || step === 'index'}
          onClick={() => void onSubmit()}
        >
          {step === 'chain'
            ? 'Signing on devnet…'
            : step === 'index'
              ? 'Indexing policy…'
              : 'Create cover on devnet'}
        </button>
      </div>
    </div>
  );
}
