import { normalizeStatus } from '@/lib/format';
import type { SettlementStatus } from '@/lib/types';

const LABELS: Record<SettlementStatus, string> = {
  PENDING: 'Pending',
  TRIGGERED: 'Triggered',
  PAID: 'Paid',
  FAILED: 'Failed',
};

export function StatusBadge({ status }: { status: string }) {
  const normalized = normalizeStatus(status);
  return (
    <span className={`badge badge-${normalized.toLowerCase()}`} data-status={normalized}>
      {LABELS[normalized]}
    </span>
  );
}
