import { API_BASE, USE_FIXTURES } from './config';
import type { OracleFeed, PolicyIndex, SettlementIndex, VerifyPayload } from './types';
import {
  FIXTURE_ORACLE,
  FIXTURE_POLICIES,
  FIXTURE_SETTLEMENTS,
  FIXTURE_VERIFY,
} from './fixtures';

export { API_BASE, USE_FIXTURES } from './config';

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

function fixtureVerify(proofHash: string): VerifyPayload | null {
  const payload = FIXTURE_VERIFY[proofHash];
  if (!payload) return null;
  return {
    ...payload,
    verified: false,
    attested: false,
    verification_method: 'fixture',
  };
}

export async function getPolicies(): Promise<{ data: PolicyIndex[]; source: 'api' | 'fixture' }> {
  try {
    const data = await fetchJson<PolicyIndex[]>('/policies');
    return { data, source: 'api' };
  } catch {
    if (!USE_FIXTURES) return { data: [], source: 'api' };
    return { data: FIXTURE_POLICIES, source: 'fixture' };
  }
}

export async function getSettlements(): Promise<{
  data: SettlementIndex[];
  source: 'api' | 'fixture';
}> {
  try {
    const data = await fetchJson<SettlementIndex[]>('/settlements');
    return { data, source: 'api' };
  } catch {
    if (!USE_FIXTURES) return { data: [], source: 'api' };
    return { data: FIXTURE_SETTLEMENTS, source: 'fixture' };
  }
}

export async function getVerify(proofHash: string): Promise<VerifyPayload | null> {
  try {
    return await fetchJson<VerifyPayload>(`/verify/${encodeURIComponent(proofHash)}`);
  } catch {
    if (!USE_FIXTURES) return null;
    return fixtureVerify(proofHash);
  }
}

export function verifyHref(proofHash: string): string {
  return `${API_BASE}/verify/${proofHash}`;
}

export async function getOracleLatest(): Promise<{
  data: OracleFeed | null;
  source: 'api' | 'fixture';
}> {
  try {
    const data = await fetchJson<OracleFeed>('/oracle/latest');
    return { data, source: 'api' };
  } catch {
    if (!USE_FIXTURES) return { data: null, source: 'api' };
    return { data: FIXTURE_ORACLE, source: 'fixture' };
  }
}
