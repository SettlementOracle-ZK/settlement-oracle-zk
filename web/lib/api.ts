import { API_BASE, USE_FIXTURES } from './config';
import { canonicalPolicyId } from './format';
import type { OracleFeed, PolicyDetail, PolicyIndex, SettlementIndex, VerifyPayload } from './types';
import {
  FIXTURE_ORACLE,
  FIXTURE_POLICIES,
  FIXTURE_POLICY_DETAILS,
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

async function fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`${API_BASE}${path}`, { cache: 'no-store', ...init });
  } catch (err) {
    const detail = err instanceof Error ? err.message : String(err);
    throw new ApiUnavailableError(
      `Cannot reach API at ${API_BASE}${path} (${detail}). From the repo root run: make db-up && cargo run --manifest-path api/Cargo.toml`,
    );
  }
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

export async function getPolicy(
  id: string,
): Promise<{ data: PolicyDetail | null; source: 'api' | 'fixture' }> {
  try {
    const data = await fetchJson<PolicyDetail>(`/policies/${encodeURIComponent(id)}`);
    return { data, source: 'api' };
  } catch {
    if (!USE_FIXTURES) return { data: null, source: 'api' };
    const data = FIXTURE_POLICY_DETAILS[canonicalPolicyId(id)] ?? null;
    return { data, source: data ? 'fixture' : 'api' };
  }
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

export type RegisterPolicyBody = {
  policy_id: string;
  holder: string;
  expiry: string;
  asset_class: string;
  policy_pda: string;
  escrow_pda: string;
  init_policy_tx?: string;
};

export async function registerPolicy(body: RegisterPolicyBody): Promise<PolicyIndex> {
  return fetchJson<PolicyIndex>('/policies/register', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
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
