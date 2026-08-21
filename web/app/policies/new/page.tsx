import { AppShell } from '@/components/AppShell';
import { CreatePolicyForm } from '@/components/CreatePolicyForm';

export default function NewPolicyPage() {
  return (
    <AppShell>
      <div className="page-head">
        <div>
          <p className="kicker">Travel · parametric</p>
          <h1>Register flight delay cover</h1>
          <p className="lede">
            Like automatic delay assist: register your flight, escrow the premium on Solana, and
            let the oracle + escrow release a payout when the delay trigger is met — no claim form.
          </p>
        </div>
      </div>
      <CreatePolicyForm />
    </AppShell>
  );
}
