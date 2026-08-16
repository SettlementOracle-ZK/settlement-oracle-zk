import type { OracleFeed, PolicyIndex, SettlementIndex, VerifyPayload } from './types';
import {
  FIXTURE_ORACLE,
  FIXTURE_POLICIES,
  FIXTURE_SETTLEMENTS,
  FIXTURE_VERIFY,
} from './fixtures';

export const API_BASE =
  process.env.NEXT_PUBLIC_API_URL?.replace(/\/$/, '') ?? 'http://127.0.0.1:3000';

export class ApiUnavailableError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ApiUnavailableError';
  }
}

async function fetchJson<T>(path: string): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, { cache: 'no-store' });
  if (!response.ok) {
    const body = await response.text();
    throw new ApiUnavailableError(`${response.status} ${path}: ${body}`);
  }
  return response.json() as Promise<T>;
}

export async function getPolicies(): Promise<{ data: PolicyIndex[]; source: 'api' | 'fixture' }> {
  try {
    const data = await fetchJson<PolicyIndex[]>('/policies');
    if (data.length === 0) {
      return { data: FIXTURE_POLICIES, source: 'fixture' };
    }
    return { data, source: 'api' };
  } catch {
    return { data: FIXTURE_POLICIES, source: 'fixture' };
  }
}

export async function getSettlements(): Promise<{
  data: SettlementIndex[];
  source: 'api' | 'fixture';
}> {
  try {
    const data = await fetchJson<SettlementIndex[]>('/settlements');
    if (data.length === 0) {
      return { data: FIXTURE_SETTLEMENTS, source: 'fixture' };
    }
    return { data, source: 'api' };
  } catch {
    return { data: FIXTURE_SETTLEMENTS, source: 'fixture' };
  }
}

export async function getVerify(proofHash: string): Promise<VerifyPayload | null> {
  try {
    return await fetchJson<VerifyPayload>(`/verify/${encodeURIComponent(proofHash)}`);
  } catch {
    return FIXTURE_VERIFY[proofHash] ?? null;
  }
}

export function verifyHref(proofHash: string): string {
  return `${API_BASE}/verify/${proofHash}`;
}

export async function getOracleLatest(): Promise<{ data: OracleFeed; source: 'api' | 'fixture' }> {
  try {
    const data = await fetchJson<OracleFeed>('/oracle/latest');
    return { data, source: 'api' };
  } catch {
    return { data: FIXTURE_ORACLE, source: 'fixture' };
  }
}
